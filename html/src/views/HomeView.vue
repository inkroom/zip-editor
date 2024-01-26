<script>
import axios from 'axios';
// import qs from 'qs';



export default {
  data(){
    return {
      data:[],
      content:'',
      path:'',
      file: this.$route.params.file,
      hasFile: this.$route.params.file!=undefined
    }
    
  },
  methods:{
    _find_parent(tree,dir,path){

      if(dir.length==1){// 最后一级，暂时认为只能是文件
        tree.push({title:dir[0],expand:false,path});
        return;
      }

        for(var i =0;i<tree.length;i++){
          if (tree[i].title == dir[0]){
            dir.shift();// 删除第一个元素
            
            this._find_parent(tree[i].children,dir,path);
            return;
          }
         
        }
         // 找完了都没有，就创建一层
         var new_tree = {title:dir[0],children:[],expand:false};
         tree.push(new_tree);
          // 继续往下
          dir.shift();
          this._find_parent(new_tree.children,dir,path);


    },

    getFileList(){
      if(this.file)
      axios.get('/api/list/'+this.file).then(res=>{

        // 排序，保证 顺序一致，因为修改文件后可能会把文件顺序放最后
       let m =  res.data.sort((a,b)=>{
        return a.name.localeCompare(b.name,'zh-CN')
        })

        // 组装成树
        let tree =[];
        res.data.forEach(m=>{
          // 拆分目录结构
          let dir = m.name.split('/');
        this. _find_parent(tree,dir,m.name)



        });

        console.log("-----")
          console.log(tree);
          this.data = tree;
    }).catch(error => {
      console.error('文件list失败', error);
      this.$Message.error('文件list失败');

    });
    },
    handleTreeClick(all,current){
      if(!current.path){
        return;
      }

      // 获取文件内容
      axios.get(`/api/file/${this.file}`,({params:{path:current.path}}))
      .then(res=>{
          this.content = res.data;
          this.path = current.path;
      }).catch(error => {
      console.error('获取文件内容失败', error);
      this.$Message.error('获取文件内容失败');

    });

    },
    handleUpload(file){
      const formData = new FormData();
      formData.append('file', file);
      console.log(file);

      axios.post('/api/upload', formData, {
      headers: {
        'Content-Type': 'multipart/form-data'
      }
    }).then(response => {
      console.log('上传成功', response.data);
      this.$Message.info('上传成功');
      this.file = file.name;
      this.hasFile = true;
      this.$router.push('/'+file.name);
      this.getFileList();

    }).catch(error => {
      console.error('上传失败', error);
      this.$Message.error('上传失败');

    });

      return false;
    },
    save(){
      axios.post(`/api/file/${this.file}`,`path=${this.path}&content=${encodeURIComponent(this.content)}`,{headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },})
    },
    download(){
      let a = document.createElement('a'); 
      a.style = 'display: none'; // 创建一个隐藏的a标签
      a.download = this.file;
      a.href = `/api/zip/${this.file}`;
      document.body.appendChild(a);
      a.click(); // 触发a标签的click事件
      document.body.removeChild(a);

    }
  }
  ,mounted(){
    console.log('home') 
    this.getFileList();
  }
}
</script>

<template>
  <div class="layout">
    <div class="tree">
      <Tree :data="data" v-if="hasFile" @on-select-change="handleTreeClick" :expand-node="true"></Tree>

    </div>

    <div class="content">
      <Row>
          <Col span="12">
            <Upload
                :before-upload="handleUpload"
                action="">
                <Button icon="ios-cloud-upload-outline">上传epub</Button>
            </Upload>
          </Col>
          <Col>
            <Affix>
              <Button type="primary" @click="save">保存</Button>
              <Button type="primary" @click="download">保存</Button>
              <!-- <a :href="'/api/zip/'+file" :download="this.file" target="_blank" class="ivu-btn ivu-btn-primary">下载</a> -->
            </Affix>
          </Col>
          
        </Row>
        <Row  v-if="content !=''" class="text">
          <textarea v-model="content"></textarea>
        </Row>
    </div>



  </div>

</template>
<style>
html,body,#app,main{
  width: 100%;
  height: 100%;
}
.content{
  height: 100%;
}
.content textarea{
  width: 98%;
}
button{
  margin: 10px !important;
}
.tree{
  position: fixed;
  top:0;
  left: 0;
  bottom: 0;
  overflow: auto;
  width: 200px;

}
.layout{
  height: 100%;
}
.content{
  margin-left: 200px;
}
.text{
  margin-top: -60px;
  padding-top: 60px;
  height: 100%;
  padding-bottom: 10px;
}
.text>textarea{
  margin-left: 20px;

  resize: none;
}
</style>
