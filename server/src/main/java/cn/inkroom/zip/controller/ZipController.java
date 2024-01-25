package cn.inkroom.zip.controller;

import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;
import lombok.extern.slf4j.Slf4j;
import net.lingala.zip4j.ZipFile;
import net.lingala.zip4j.exception.ZipException;
import net.lingala.zip4j.io.inputstream.ZipInputStream;
import net.lingala.zip4j.model.FileHeader;
import net.lingala.zip4j.model.ZipParameters;
import org.apache.tomcat.util.http.fileupload.IOUtils;
import org.springframework.util.StringUtils;
import org.springframework.web.bind.annotation.*;
import org.springframework.web.multipart.MultipartFile;

import java.io.*;
import java.nio.charset.StandardCharsets;
import java.util.*;
import java.util.function.Function;

@Slf4j
@RestController
@RequestMapping("/api/")
public class ZipController {

    @PostMapping("upload")
    public String upload(MultipartFile file) throws IOException {
        String currentDirectory = System.getProperty("user.dir");
        log.info("begin {} {}", file.getName(), currentDirectory);

        file.transferTo(new File(currentDirectory + "/zip/" + file.getOriginalFilename()));
        log.info("end {}", file.getName());

        return "ok";
    }

    @GetMapping("zip/{file}")
    public void list(@PathVariable("file") String file, HttpServletRequest request, HttpServletResponse response) throws IOException {
        String currentDirectory = System.getProperty("user.dir");

        String fileName = file;
        String userAgent = request.getHeader("User-Agent"); //针对IE或者以IE为内核的浏览器：
        if (StringUtils.hasLength(userAgent) && (userAgent.contains("MSIE") || userAgent.contains("Trident"))) {
            fileName = java.net.URLEncoder.encode(fileName, StandardCharsets.UTF_8);
        } else { //非IE浏览器的处理：
            fileName = new String(fileName.getBytes(StandardCharsets.UTF_8), StandardCharsets.ISO_8859_1);
        }
        response.setHeader("Content-disposition", String.format("attachment; filename=\"%s\"", fileName));
        response.setCharacterEncoding("UTF-8");

        IOUtils.copy(new FileInputStream(currentDirectory + "/zip/" + file), response.getOutputStream());

    }

    @GetMapping("list/{file}")
    public List<Map<String, Object>> list(@PathVariable("file") String file) throws IOException {

        String currentDirectory = System.getProperty("user.dir");
        List<Map<String, Object>> res = new ArrayList<>();
        try (ZipFile zip = new ZipFile(currentDirectory + "/zip/" + file);) {

            List<FileHeader> fileHeaders = zip.getFileHeaders();
            res.addAll(fileHeaders.stream().map(new Function<FileHeader, Map<String, Object>>() {
                @Override
                public Map<String, Object> apply(FileHeader fileHeader) {
                    Map<String, Object> m = new HashMap<>();
                    m.put("name", fileHeader.getFileName());
                    m.put("isDir", fileHeader.isDirectory());

                    return m;
                }
            }).toList());
        }

        return res;
    }

    @GetMapping("file/{file}")
    public String content(@PathVariable("file") String file, @RequestParam("path") String path) throws IOException {

        String currentDirectory = System.getProperty("user.dir");
        List<Map<String, Object>> res = new ArrayList<>();
        try (ZipFile zip = new ZipFile(currentDirectory + "/zip/" + file);) {
            FileHeader fileHeader = zip.getFileHeader(path);
            ZipInputStream inputStream = zip.getInputStream(fileHeader);
            ByteArrayOutputStream arr = new ByteArrayOutputStream();
            IOUtils.copy(inputStream, arr);
            return arr.toString(StandardCharsets.UTF_8);
        }
    }

    @PostMapping("file/{file}")
    public String content(@PathVariable("file") String file, @RequestParam("path") String path, @RequestParam("content") String content) {
        String currentDirectory = System.getProperty("user.dir");
        try (ZipFile zip = new ZipFile(currentDirectory + "/zip/" + file);) {

            FileHeader fileHeader = zip.getFileHeader(path);


            ZipParameters zipParameters = new ZipParameters();
            zipParameters.setFileNameInZip(path);
            zipParameters.setLastModifiedFileTime(fileHeader.getLastModifiedTime());
            Optional.ofNullable(fileHeader.getAesExtraDataRecord()).ifPresent(z -> {
                zipParameters.setAesVersion(z.getAesVersion());
                zipParameters.setAesKeyStrength(z.getAesKeyStrength());
            });
            zipParameters.setEntryCRC(fileHeader.getCrc());
            zipParameters.setEntrySize(fileHeader.getFileNameLength());
            zipParameters.setFileComment(fileHeader.getFileComment());
            zipParameters.setCompressionMethod(fileHeader.getCompressionMethod());
            zipParameters.setEncryptionMethod(fileHeader.getEncryptionMethod());


            zip.addStream(new ByteArrayInputStream(content.getBytes(StandardCharsets.UTF_8)), zipParameters);

        } catch (ZipException e) {
            throw new RuntimeException(e);
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
        return "ok";
    }
}
