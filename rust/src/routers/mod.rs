use salvo::{
    hyper::body::Bytes, prelude::{CatchPanic, Logger}, Router
};
use salvo::http::StatusCode;
use salvo::writing::Text;
use salvo::Response;
use zip::write::FileOptions;
use std::{collections::HashMap, io::{Read, Seek, Write}, path::Path, ptr::read};
use salvo::handler;
use salvo::Request;
use rust_embed::RustEmbed;
use salvo::serve_static::static_embed;
use serde::{Serialize, Deserialize};

const EXTRACT_DIR: &str = "extract";
const TEMP_DIR:&str = "temp";
const EPUB_DIR:&str = "epub";

#[derive(RustEmbed)]
#[folder = "static"]
struct Assets;

#[derive(Serialize, Deserialize, Debug)]
struct Progress{
    file:String,
    path:String
}

#[derive(Serialize, Deserialize, Debug)]
struct ListProgress{
    
    path:String,
    list:Vec< HashMap<String,String>>
}
pub fn router() -> Router {
    // let _cors_handler = cors_middleware();
    // let mut static_routers = static_routers::create_static_routers();
    let router = Router::new()
        //.hoop(_cors_handler)
        .hoop(Logger::new())
        .hoop(CatchPanic::new())
        .push(
            Router::with_path("/api")
                    .push(Router::with_path("/upload").post(upload))
                    .push(Router::with_path("/list/<file>").get(list))
                    .push(Router::with_path("/file/<file>").get(content).post(save))
                    .push(Router::with_path("/assets/<file>").get(content))
                    .push(Router::with_path("/zip/<file>").get(download))
                    .push(Router::with_path("/dir_list").get(dir_list))

                )
                .push(Router::with_path("<**path>").get(static_embed::<Assets>().fallback("index.html")))

        
    
        // .get(hello)
        // .append(&mut static_routers)
        ;
        router
    // let doc = OpenApi::new("salvo web api", "0.0.1").merge_router(&router);
    // router
    //     .push(doc.into_router("/api-doc/openapi.json"))
    //     .push(SwaggerUi::new("/api-doc/openapi.json").into_router("swagger-ui"))
}


#[handler]
async fn upload(req: &mut Request, res: &mut Response) {
    let files = req.files("file").await;
    std::fs::create_dir_all(TEMP_DIR).unwrap();
    std::fs::create_dir_all(EXTRACT_DIR).unwrap();
    if let Some(files) = files {
        let mut msgs = Vec::with_capacity(files.len());
        for file in files {
            let dest = format!("{}/{}",TEMP_DIR, file.name().unwrap_or("file"));
            if let Err(e) = std::fs::copy(&file.path(), Path::new(&dest)) {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Text::Plain(format!("file not found in request: {}", e)));
            } else {
               

                // 解压文件 ， 因为 zip 库不支持修改单个文件，所以只能解压后操作
                let prefix = format!("{}/{}.dir",EXTRACT_DIR,file.name().unwrap_or("file"));
                println!("prefix = {}",prefix);
                extract(dest.as_str(), prefix.as_str());
                msgs.push(dest);
            }
        }
        res.render(Text::Plain(format!(
            "Files uploaded:\n\n{}",
            msgs.join("\n")
        )));
    } else {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Text::Plain("file not found in request"));
    }
}

#[handler]
async fn list(req: &mut Request, res: &mut Response){
    let file_name = req.param::<String>("file").unwrap();

    println!("file = {}",file_name);

    let abpath =format!("{}/{}",TEMP_DIR,file_name);


    let mut fname = std::path::Path::new(&abpath);

    if !fname.exists() {
        
        fname = std::path::Path::new(&file_name);

    }

    let file = std::fs::File::open(fname).unwrap();
    let reader = std::io::BufReader::new(file);

    let mut result:  Vec<HashMap<String,String>> = Vec::new();
    let mut archive = zip::ZipArchive::new(reader).unwrap();

    for i in 0..archive.len() {
        let file = archive.by_index(i).unwrap();
        println!("file name ={}",file.name());
        result.push(HashMap::from([( String::from("name") ,String::from(file.name()))]));

    }

    // 获取进度
    let mut path = String::new();
    std::fs::create_dir_all(EPUB_DIR).unwrap_or_else(|why|{
        println!("! {:?}", why.kind());
    });
    if let Ok(mut input) =std::fs::OpenOptions::new().write(true).create(true).read(true).open(format!("{}/temp.progress",EPUB_DIR) ) {
        let mut buf = String::new();
        input.read_to_string(&mut buf).unwrap();
        
        if buf.len() != 0{
                    // 反序列化
        let progress:Vec<Progress> = serde_json::from_str(&buf).unwrap();
        for ele in progress {
            if ele.file == file_name {
                path = ele.path;
            }
        }
        }

         


    }


    let out_result = ListProgress{
        path,
        list:result
    };
    let out = serde_json::to_string(&out_result).unwrap();


    res.add_header("content-type", "application/json; charset=utf-8", true).unwrap();
    // res.render(salvo::prelude::Json(res));
    res.render(out);
    
}
/**
 * 写入进度
 */
fn write_progress(file:&str,path:&str){

    if !path.ends_with(".xhtml") {
        return;
    }
    std::fs::create_dir_all(EPUB_DIR).unwrap_or_else(|why|{
        println!("! {:?}", why.kind());
    });
    if let Ok(mut input) = std::fs::OpenOptions::new().write(true).create(true).read(true).open(format!("{}/temp.progress",EPUB_DIR)) {
        let mut buf = String::new();
        input.read_to_string(&mut buf).unwrap();
        let mut progress:Vec<Progress>;
        if buf.len()==0{
            progress=Vec::new();
        }else{
            // 反序列化
             progress = serde_json::from_str(&buf).unwrap();
        }
        let mut find =false;
        for ele in &mut progress {
            if ele.file == file {
                ele.path = String::from(path);
                find = true;
                break;
            }
        }
        if !find{
            progress.push(Progress { file: String::from(file), path: String::from(path) });
        }

        let res = serde_json::to_string(&progress).unwrap();
        println!("out  = {}",res);
        let m = res.as_bytes();
        input.seek(std::io::SeekFrom::Start(0)).unwrap();// 把指针定到开头，否则会从读到的最后一个字节开始写
        input.write_all(m).unwrap();
        input.flush().unwrap();
        input.set_len(m.len() as u64).unwrap();// 由于 新内容可能比旧内容短，所以要重新设置长度

    }
}

#[handler]
async fn content(req: &mut Request, res: &mut Response){
    let file = req.param::<String>("file").unwrap();
    let path =req.query::<String>("path").unwrap();

    let abpath =format!("{}/{}.dir/{}",EXTRACT_DIR,file,path);

    let mut fname = std::path::Path::new(&abpath);
    write_progress(file.as_str(), path.as_str());
    if fname.exists() {
        
        // fname = std::path::Path::new(&file);
        // 展开文件夹方式
        let mut ffile = std::fs::File::open(fname).unwrap();

        let mut out = Vec::new();
        let s =ffile.read_to_end(&mut out);
        if s.is_err(){
            _= res.write_body(s.err().unwrap().to_string());
            res.status_code(StatusCode::NOT_FOUND);

        }else {
            res.write_body(Bytes::from(out)).unwrap();

        }
        
        return ;
    }
    fname = std::path::Path::new(&file);
    // 直接读文件方式
    println!("fname {} path ={}",fname.display(),path);

    let file = std::fs::File::open(fname).unwrap();
    let reader = std::io::BufReader::new(file);

    let mut archive: zip::ZipArchive<std::io::BufReader<std::fs::File>> = zip::ZipArchive::new(reader).unwrap();
    let mut out = Vec::new();

    let mut ff =     archive.by_name(path.as_str());
    if ff.is_err(){
        let r = String::from(format!("{:?}",ff.err().unwrap()));
        res.write_body(Bytes::from(Vec::from(r.as_bytes()))).unwrap();
        res.status_code(StatusCode::NOT_FOUND);
        return ;
    }
    ff.expect("3223").read_to_end(&mut out).expect("ssss");
    res.write_body(Bytes::from(out)).expect("3823");
    return ;
     
}



#[handler]
async fn save(req: &mut Request, res: &mut Response){
    let file = req.param::<String>("file").unwrap();
    let path = req.form::<String>("path").await.unwrap();
    let file_content = req.form::<String>("content").await.unwrap();
    println!("op {} file=[{}]",path,file);

    // let abpath =format!("temp/{}",file);
    let abpath =format!("{}/{}.dir/{}",EXTRACT_DIR,file,path);

    let mut fname = std::path::Path::new(&abpath);

    if !fname.exists() {
        
        fname = std::path::Path::new(&file);

    }
    let mut file =  std::fs::OpenOptions::new().write(true).truncate(true).read(true).open(fname).unwrap(); 
    
    file.write_all(file_content.as_bytes()).unwrap();
    file.flush().unwrap();

    // std::io::copy(&mut file_content,&mut file);


    // let reader = std::io::BufReader::new(&file);

    // let mut archive = zip::ZipArchive::new(reader).unwrap();
    
    // let mut found_file =None;

    // if let Ok(s) = archive.by_name(path.as_str()) {
    //     found_file = Some(s);
    // }
    

    // if found_file.is_none() {
        
    //     res.status_code(StatusCode::NOT_FOUND);
    //     return ;
    // }
    // let ff = found_file.unwrap();
    // let write_file =  std::fs::OpenOptions::new().write(true).read(true).open(fname).unwrap(); 
    // let mut zip = zip::ZipWriter::new(&file);
    // let option = FileOptions::default()
    // // .compression_method(ff.compression())
    // // .last_modified_time(ff.last_modified())
    // // .large_file(false)
    // // .unix_permissions(ff.unix_mode().unwrap())
    
    // ;
    // zip.start_file(path, option).unwrap();
    // zip.write_all(file_content.as_bytes()).unwrap();
    // zip.flush().unwrap();
    // let mut w = zip.finish().unwrap();
    // // 因为修改完后文件长度可能小于原文件，所以需要重新设置长度
    // w.set_len(w.stream_position().unwrap()).unwrap();

    // zip.write_vectored(bufs)

    res.status_code(StatusCode::OK);
}

#[handler]
async fn download(req: &mut Request, res: &mut Response){
    let file = req.param::<String>("file").unwrap();
    let abpath =format!("{}/{}.dir/",EXTRACT_DIR,file);
    let dest = format!("{}/{}",TEMP_DIR,file);
    create_zip(abpath.as_str(), dest.as_str(), zip::CompressionMethod::Zstd).unwrap();

    res.send_file(std::path::Path::new(&dest), req.headers()).await;
}

#[handler]
async fn dir_list(req: &mut Request, res: &mut Response){

    let walkdir = walkdir::WalkDir::new(EPUB_DIR);
    let it = walkdir.into_iter();
    let mut result:  Vec<HashMap<String,String>> = Vec::new();

    for e in it.filter_map(|e| e.ok()) {
        result.push(HashMap::from([( String::from("name") ,e.path().display().to_string())]));
    }
    

    let out = serde_json::to_string(&result).unwrap();
    res.add_header("content-type", "application/json; charset=utf-8", true).unwrap();
    // res.render(salvo::prelude::Json(res));
    res.render(out);

}

fn zip_dir<T>(
    it: &mut dyn Iterator<Item = walkdir::DirEntry>,
    prefix: &str,
    writer: T,
    method: zip::CompressionMethod,
) -> zip::result::ZipResult<()>
where
    T: Write + Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix)).unwrap();

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            // println!("adding file {path:?} as {name:?} ...");
            #[allow(deprecated)]
            zip.start_file_from_path(name, options)?;
            let mut f = std::fs::File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            // println!("adding dir {path:?} as {name:?} ...");
            #[allow(deprecated)]
            zip.add_directory_from_path(name, options)?;
        }
    }
    zip.finish()?;
    Result::Ok(())
}

fn create_zip(
    src_dir: &str,
    dst_file: &str,
    method: zip::CompressionMethod,
) -> zip::result::ZipResult<()> {

    let path = Path::new(dst_file);
    let file = std::fs::File::create(path).unwrap();

    let walkdir = walkdir::WalkDir::new(src_dir);
    let it = walkdir.into_iter();

    zip_dir(&mut it.filter_map(|e| e.ok()), src_dir, file, method)?;

    Ok(())
}


fn extract(path:&str,prefix:&str) -> i32 {

    let fname = std::path::Path::new(path);
    let file = std::fs::File::open(fname).unwrap();

    let mut archive = zip::ZipArchive::new(file).unwrap();
    

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let mut outpath = match file.enclosed_name() {
            Some(path) => path,
            None => continue,
        };
        let temp = format!("{}/{}",prefix,outpath.display());
        outpath = std::path::Path::new(&temp);


        if (*file.name()).ends_with('/') {
            // println!("File {} extracted to \"{:?}\"", i, outpath);
            std::fs::create_dir_all(&outpath).unwrap();
        } else {
            // println!(
            //     "File {} extracted to \"{}\" ({} bytes)",
            //     i,
            //     outpath.display(),
            //     file.size()
            // );
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p).unwrap();
                }
            }
            let mut outfile = std::fs::File::create(&outpath).unwrap();
            std::io::copy(&mut file, &mut outfile).unwrap();
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode)).unwrap();
            }
        }
    }

    0
}


#[cfg(test)]
mod tests {
    use salvo::prelude::*;
    use salvo::test::{ResponseExt, TestClient};

    use crate::config::CFG;

    #[tokio::test]
    async fn test_write_less_file() {
        use std::io::Write;
        let abpath =format!("test.txt");

        {

            let file_content = "1234";

        
            let mut fname = std::path::Path::new(&abpath);
    
            let mut file =  std::fs::OpenOptions::new().write(true).create(true).truncate(true).read(true).open(fname).unwrap(); 
            let len = file_content.as_bytes();
            file.write_all(len).unwrap();
            file.flush().unwrap();
        }
        // file.set_len(len.len() as usize);

        {

            let file_content = "abc";

        
            let mut fname = std::path::Path::new(&abpath);
    
            let mut file =  std::fs::OpenOptions::new().write(true).truncate(true).read(true).open(fname).unwrap(); 
            let len = file_content.as_bytes();
            // file.set_len(0);
            // file.write_all(len).unwrap();
            // file.flush().unwrap();
        }


    }
}