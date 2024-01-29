<script>
import axios from 'axios';

var resolvePath = function () {
  function resolve(pathA, pathB) {
    // 先做split，得到的结果如下几种
    //  ‘a’     => ['a']
    //  'a/b'   => ['a', 'b']
    //  '/a/b'  => ['', 'a', 'b']
    //  '/a/b/' => ['', 'a', 'b', '']
    pathB = pathB.split('/');
    if (pathB[0] === '') {
      // 如果pathB是想对于根目录
      // 则不在考虑pathA，直接返回pathB
      return pathB.join('/');
    }
    pathA = pathA.split('/');
    var aLastIndex = pathA.length - 1;
    if (pathA[aLastIndex] !== '') {
      // 文件名出栈，只保留路径
      pathA[aLastIndex] = '';
    }

    var part;
    var i = 0;
    while (typeof (part = pathB[i]) === 'string') {
      switch (part) {
        case '..':
          // 进入父级目录
          pathA.pop();
          pathA.pop();
          pathA.push('');
          break;
        case '.':
          // 当前目录
          break;
        default:
          // 进入子目录
          pathA.pop();
          pathA.push(part);
          pathA.push('');
          break;
      }
      i++;
    }
    return pathA.join('/');
  }

  var paths = arguments;
  var i = 0;
  var path;
  var r = location.pathname;
  var multiSlashReg = /\/\/+/g;
  while (typeof (path = paths[i]) === 'string') {
    // '//' ==> '/'
    path = path.replace(multiSlashReg, '/');
    r = resolve(r, path);
    i++;
  }

  return r;
};

var areaScrollTop = {};
var previewScrollTop = {};
export default {
  data() {
    return {
      data: [],
      content: '',
      path: '',
      file: this.$route.params.file,
      hasFile: this.$route.params.file != undefined,
      isPreview: false,
      previewContent: ''
    }

  },
  methods: {
    _find_parent(tree, dir, path) {

      if (dir.length == 1) {// 最后一级，暂时认为只能是文件
        tree.push({ title: dir[0], expand: false, path });
        return;
      }

      for (var i = 0; i < tree.length; i++) {
        if (tree[i].title == dir[0]) {
          dir.shift();// 删除第一个元素

          this._find_parent(tree[i].children, dir, path);
          return;
        }

      }
      // 找完了都没有，就创建一层
      var new_tree = { title: dir[0], children: [], expand: false };
      tree.push(new_tree);
      // 继续往下
      dir.shift();
      this._find_parent(new_tree.children, dir, path);


    },

    getFileList() {
      if (this.file)
        axios.get('/api/list/' + this.file).then(res => {

          // 排序，保证 顺序一致，因为修改文件后可能会把文件顺序放最后
          let m = res.data.sort((a, b) => {
            return a.name.localeCompare(b.name, 'zh-CN')
          })

          // 组装成树
          let tree = [];
          res.data.forEach(m => {
            // 拆分目录结构
            let dir = m.name.split('/');
            this._find_parent(tree, dir, m.name)



          });

          this.data = tree;
        }).catch(error => {
          console.error('文件list失败', error);
          this.$Message.error('文件list失败');

        });
    },
    handleTreeClick(all, current) {
      if (!current) {
        current = all;
      }
      if (!current.path) {
        return;
      }
      if (this.$refs.text)
        // 记录高度
        areaScrollTop[this.file + "/" + this.path] = this.$refs.text.scrollTop
      if (this.$refs.preview) {
        var iframe = this.$refs.preview,
          iframedoc = iframe.contentDocument || iframe.contentWindow.document;
        previewScrollTop[this.file + '/' + this.path] = iframedoc.documentElement.scrollTop;
      }
      // 获取文件内容
      axios.get(`/api/file/${this.file}`, ({ params: { path: current.path } }))
        .then(res => {
          this.content = res.data;
          this.path = current.path;
          // 重新设置滚动位置
          if (areaScrollTop[this.file + "/" + this.path]) {
            this.$nextTick(() => {
              this.$refs.text.scrollTop = areaScrollTop[this.file + "/" + this.path];
            })
          }


          this.getPreviewContent();
        }).catch(error => {
          console.error('获取文件内容失败', error);
          this.$Message.error('获取文件内容失败');

        });

    },
    renderTreeContent(h, { root, node, data }) {
      return h('span', {
        style: {
          display: 'inline-block',
          float: 'right',
          marginRight: '32px',
          fontWeight: data.path == this.path ? '800' : 400
        },
        innerHTML: data.title,
        onClick: () => { this.handleTreeClick(data) }
      }
      );
    },
    handleUpload(file) {
      const formData = new FormData();
      formData.append('file', file);
      this.$Spin.show();
      axios.post('/api/upload', formData, {
        headers: {
          'Content-Type': 'multipart/form-data'
        }
      }).then(response => {
        console.log('上传成功', response.data);
        this.$Message.info('上传成功');
        this.file = file.name;
        this.hasFile = true;
        this.$router.push('/' + file.name);
        this.getFileList();

      }).catch(error => {
        console.error('上传失败', error);
        this.$Message.error('上传失败');

      }).finally(s => {
        this.$Spin.hide();
      });

      return false;
    },
    save() {
      axios.post(`/api/file/${this.file}`, `path=${this.path}&content=${encodeURIComponent(this.content)}`, {
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
        },
      })
    },
    download() {
      let a = document.createElement('a');
      a.style = 'display: none'; // 创建一个隐藏的a标签
      a.download = this.file;
      a.href = `/api/zip/${this.file}`;
      document.body.appendChild(a);
      a.click(); // 触发a标签的click事件
      document.body.removeChild(a);

    },

    getPreviewContent() {
      if (this.isPreview) {
        // 处理外部依赖
        // let p = document.createElement('div');
        let p = new DOMParser().parseFromString(this.content, 'text/html');

        p.innerHTML = this.content;
        let imgs = p.getElementsByTagName('img');
        for (let i = 0; i < imgs.length; i++) {
          let img = imgs[i];
          let src = img.getAttribute('src')
          if (src.startsWith('http://') || src.startsWith('https://') || src.startsWith('//'))
            continue;
          let new_src = resolvePath(this.path.substring(0, this.path.lastIndexOf('/')), src);
          // 去除开头结尾的 /
          new_src = new_src.substring(1, new_src.length - 1);

          img.setAttribute('src', location.protocol + "//" + location.host + "/api/assets/" + this.file + "?path=" + encodeURIComponent(new_src));

        }

        let links = p.getElementsByTagName('link');
        for (let i = 0; i < links.length; i++) {
          let link = links[i];
          let src = link.getAttribute('href')
          if (src.startsWith('http://') || src.startsWith('https://') || src.startsWith('//'))
            continue;
          let new_src = resolvePath(this.path.substring(0, this.path.lastIndexOf('/')), src);
          // 去除开头结尾的 /
          new_src = new_src.substring(1, new_src.length - 1);

          link.setAttribute('href', location.protocol + "//" + location.host + "/api/assets/" + this.file + "?path=" + encodeURIComponent(new_src));

        }
        const serializer = new XMLSerializer();
        const xmlStr = serializer.serializeToString(p);
        this.$nextTick(() => {
          // p.innerHTML 的img没有修改，p.body.innerHTML里没有head部分
          this.previewContent = xmlStr;
          var iframe = this.$refs.preview;
          if (!iframe) return;
          var iframedoc = iframe.contentDocument || iframe.contentWindow.document;
          // iframedoc.body.innerHTML = this.previewContent;// 这个内容闪一下就没了

          // console.log(this.previewContent)
          iframedoc.open();
          iframedoc.write(this.previewContent);
          iframedoc.close();
          if (previewScrollTop[this.file + "/" + this.path])
            this.$nextTick(() => {
              iframedoc.documentElement.scrollTop = previewScrollTop[this.file + "/" + this.path];
            })
        })
      }
    },

    toggle() {
      if (this.isPreview) {
        this.previewContent = '';
      }
      this.isPreview = !this.isPreview
      this.$nextTick(() => {
        this.getPreviewContent();
      })

    }
  }
  , mounted() {
    console.log('home')
    this.getFileList();
  }
}
</script>

<template>
  <div class="layout">
    <div class="tree">
      <Tree :data="data" v-if="hasFile" @on-select-change="handleTreeClick" :expand-node="true"
        @on-toggle-expand="handleTreeClick" :render="renderTreeContent"></Tree>

    </div>

    <div class="content">
      <Row>
        <Col span="12">
        <Upload style="display: inline-block;" :before-upload="handleUpload" action="">
          <Button icon="ios-cloud-upload-outline">上传epub</Button>
        </Upload>
        <Button @click="toggle">切换</Button>

        </Col>
        <Col>
        <Affix>
          <Button type="primary" @click="save">保存</Button>
          <Button type="primary" @click="download">保存</Button>
          <!-- <a :href="'/api/zip/'+file" :download="this.file" target="_blank" class="ivu-btn ivu-btn-primary">下载</a> -->
        </Affix>
        </Col>

      </Row>
      <Row v-if="content != ''" class="text">
        <textarea v-model="content" v-if="!isPreview" ref="text"></textarea>
        <iframe v-else class="preview" ref="preview"></iframe>
        <!-- <div v-else v-html="previewContent" class="preview"></div> -->
      </Row>
    </div>



  </div>
</template>
<style>
html,
body,
#app,
main {
  width: 100%;
  height: 100%;
}

.content {
  height: 100%;
}

.content textarea {
  width: 98%;
}

button {
  margin: 10px !important;
}

.tree {
  position: fixed;
  top: 0;
  left: 0;
  bottom: 0;
  overflow: auto;
  width: 200px;

}

.layout {
  height: 100%;
}

.content {
  margin-left: 200px;
}

.text {
  margin-top: -60px;
  padding-top: 60px;
  height: 100%;
  padding-bottom: 10px;
}

.text>textarea {
  margin-left: 20px;

  resize: none;
}

.preview {
  margin: 20px;
  width: 100%;
}</style>
