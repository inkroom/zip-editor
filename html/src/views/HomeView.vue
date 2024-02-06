<script>
import axios from 'axios';
import { NButton, NUpload, NTree, useMessage, NSpin } from 'naive-ui';
import { h } from 'vue';

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
  components: { NButton, NUpload, NTree, NSpin },
  data() {

    let _this = this;
    return {
      message: useMessage(),
      data: [],
      loading: false,
      originData: [],
      content: '',
      path: '',
      file: this.$route.params.file,
      hasFile: this.$route.params.file != undefined,
      isPreview: false,
      previewContent: '',
      lastExpandData: undefined,
      hide: false,
      expandedKeys: ['expand'],
      nodeProps: ({ option }) => {
        return {
          onClick() {
            let data = option;
            if (data.children) {
              if (_this.lastExpandData && _this.lastExpandData != data && _this.lastExpandData.level == data.level) {
                // 展开互斥，一次只能展开一个节点
                _this.lastExpandData.expand = false;
              }
              _this.lastExpandData = data;
            }
            if (!option.children) {
              option.expand = true;
            }
            _this.handleTreeClick(option)
          }
        }
      }
    }

  },
  methods: {
    _build_tree(tree, dir, path, parent, level) {

      if (!level) level = 1;
      // debugger
      // console.log('tree dir path parent',tree,dir,path,parent);
      if (dir.length == 1) {// 最后一级，暂时认为只能是文件
        tree.push({ title: dir[0], expand: false, path, level });
        return;
      }

      for (var i = 0; i < tree.length; i++) {
        if (tree[i].title == dir[0]) {
          // console.log('2 tree dir path parent',tree,dir,path,parent);

          dir.shift();// 删除第一个元素
          // console.log('3 tree dir path parent',tree,dir,path,parent);

          this._build_tree(tree[i].children, dir, path, tree[i].path + "/", level + 1);
          return;
        }

      }
      // 找完了都没有，就创建一层
      var new_tree = { title: dir[0], children: [], expand: false, path: parent + dir[0], level };
      tree.push(new_tree);
      // 继续往下
      let p = dir.shift();
      this._build_tree(new_tree.children, dir, path, parent + p + '/', level + 1);


    },

    getFileList() {
      if (this.file)
        axios.get('/api/list/' + encodeURIComponent(this.file)).then(res => {

          // 排序，保证 顺序一致，因为修改文件后可能会把文件顺序放最后
          let sorted = res.data.list.sort((a, b) => {
            return a.name.localeCompare(b.name, 'zh-CN')
          })

          // 组装成树
          let tree = [];
          sorted.forEach(m => {
            // 拆分目录结构
            let dir = m.name.split('/');
            this._build_tree(tree, dir, m.name, "")



          });

          this.originData = res.data.list;
          this.data = tree;
          if (res.data.path != '') {
            this.path = res.data.path;

            this.seek(0);

          }
        }).catch(error => {
          console.error('文件list失败', error);
          this.message.error('文件list失败');

        });
    },
    handleTreeClick(all, current) {
      if (!current) {
        current = all;
      }
      if (!current.path) {
        return;
      }
      if (current.children) {
        return 'toggleExpand';
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
      axios.get(`/api/file/${encodeURIComponent(this.file)}`, ({ params: { path: current.path } }))
        .then(res => {
          this.content = res.data;
          this.path = current.path;
          // 重新设置滚动位置

          this.$nextTick(() => {
            if (this.$refs.text) {
              if (areaScrollTop[this.file + "/" + this.path]) {
                this.$refs.text.scrollTop = areaScrollTop[this.file + "/" + this.path];
              } else {
                this.$refs.text.scrollTop = 0
              }
            }

          })



          this.getPreviewContent();
        }).catch(error => {
          console.error('获取文件内容失败', error);
          this.message.error('获取文件内容失败');

        });

    },
    renderTreeContent({ option }) {
      let data = option;
      return h('span', {
        style: {
          display: 'inline-block',
          float: 'right',
          marginRight: '32px',
          fontWeight: (data.path == this.path || this.path.startsWith(data.path)) ? '800' : 400
        },
        innerHTML: data.title,
      });
    },
    handleUpload(file) {
      if (file.event != 'change') {
        return false;
      }
      file = file.file;

      const formData = new FormData();
      formData.append('file', file);
      this.loading = true;
      axios.post('/api/upload', formData, {
        headers: {
          'Content-Type': 'multipart/form-data'
        }
      }).then(response => {
        console.log('上传成功', response.data);
        this.message.info('上传成功');
        this.file = file.name;
        this.hasFile = true;
        this.$router.push('/' + file.name);
        this.getFileList();

      }).catch(error => {
        console.error('上传失败', error);
        this.message.error('上传失败');

      }).finally(s => {
        this.loading = false;
      });

      return false;
    },
    beforeUpload() {
      this.loading = true;
    },
    uploadSuccess({ file }) {
      this.message.info('上传成功');
      this.file = file.name;
      this.hasFile = true;
      this.$router.push('/' + file.name);
      this.getFileList();
      this.loading = false;
    },
    uploadError() {
      console.error('上传失败', error);
      this.message.error('上传失败');
      this.loading = false;
    },
    save() {
      axios.post(`/api/file/${encodeURIComponent(this.file)}`, `path=${this.path}&content=${encodeURIComponent(this.content)}`, {
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

          img.setAttribute('src', location.protocol + "//" + location.host + "/api/assets/" + encodeURIComponent(this.file) + "?path=" + encodeURIComponent(new_src));

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

          this.$nextTick(() => {
            if (previewScrollTop[this.file + "/" + this.path])
              iframedoc.documentElement.scrollTop = previewScrollTop[this.file + "/" + this.path];
            else {
              iframedoc.documentElement.scrollTop = 0;
            }
            iframedoc.documentElement.style = 'color: #afb5c2'
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

    },
    eclpise() {

      this.hide = !this.hide;

    },

    seek(sep) {
      let f = this.path;
      let index = this.originData.findIndex(s => s.name == f);
      if (index != -1) {
        index += sep;
        if (index == this.originData.length) return;
        if (index == -1) return;
        this.path = this.originData[index].name;

        // 找到树节点
        let find_in_tree = function (tree, path) {
          if (Array.isArray(tree)) {
            for (let i = 0; i < tree.length; i++) {
              let s = find_in_tree(tree[i], path);
              if (s != null) {
                return s;
              }
            }
            return null;
          }

          if (tree.path == path) return tree;
          if (tree.children) {
            for (let i = 0; i < tree.children.length; i++) {
              let s = find_in_tree(tree.children[i], path);
              if (s != null) {
                return s;
              }
            }
          }
          return null;
        }
        let s = find_in_tree(this.data, this.path)
        if (s == null) return;
        this.handleTreeClick(s);
      }

    },
    prev() {
      // 上一个
      this.seek(-1);
    },
    next() {
      this.seek(1);
    }
  }
  , mounted() {
    console.log('home')
    this.getFileList();
  }
}
</script>

<template>
  <div class="body">
    <div class="left tree" v-if="!hide">

      <n-tree :data="data" v-if="hasFile" @on-select-change="handleTreeClick" :expand-on-click="true"
        :node-props="nodeProps" key-field="path" label-field="title" @on-toggle-expand="handleTreeClick"
        :render-label="renderTreeContent"></n-tree>

    </div>

    <div :class="hide ? 'right-content' : 'right-content hide'">
      <n-spin :show="loading">
        <div class="header">

          <n-button type="primary" @click="eclpise" quaternary>
            <!-- 折叠，也就是精简 -->
            <template #icon>
              <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 512 512">
                <circle cx="256" cy="256" r="26" fill="currentColor"></circle>
                <circle cx="346" cy="256" r="26" fill="currentColor"></circle>
                <circle cx="166" cy="256" r="26" fill="currentColor"></circle>
                <path d="M448 256c0-106-86-192-192-192S64 150 64 256s86 192 192 192s192-86 192-192z" fill="none"
                  stroke="currentColor" stroke-miterlimit="10" stroke-width="32"></path>
              </svg>
            </template>
          </n-button>
          <n-button @click="toggle" quaternary>
            <template #icon>
              <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 24 24">
                <g fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M15 4h4v4"></path>
                  <path d="M14.75 9.25L19 4"></path>
                  <path d="M5 19l4-4"></path>
                  <path d="M15 19h4v-4"></path>
                  <path d="M5 5l14 14"></path>
                </g>
              </svg>
            </template>
          </n-button>
          <n-upload style="display: inline-block;" :on-finish="uploadSuccess" :show-file-list="false"
            :on-before-upload="beforeUpload" :on-error="uploadError" action="/api/upload" v-if="!hide" :loading="loading">
            <!-- 上传 -->
            <n-button quaternary>
              <template #icon>
                <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 512 512">
                  <path
                    d="M320 367.79h76c55 0 100-29.21 100-83.6s-53-81.47-96-83.6c-8.89-85.06-71-136.8-144-136.8c-69 0-113.44 45.79-128 91.2c-60 5.7-112 43.88-112 106.4s54 106.4 120 106.4h56"
                    fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="32">
                  </path>
                  <path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="32"
                    d="M320 255.79l-64-64l-64 64"></path>
                  <path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="32"
                    d="M256 448.21V207.79"></path>
                </svg>
              </template>
            </n-button>
          </n-upload>


          <n-button type="primary" @click="save" quaternary v-if="!hide">
            <!-- 保存文本 -->
            <template #icon>
              <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 24 24">
                <g fill="none">
                  <path
                    d="M3 5.75A2.75 2.75 0 0 1 5.75 3h9.964a3.25 3.25 0 0 1 2.299.952l2.035 2.035c.61.61.952 1.437.952 2.299v9.964A2.75 2.75 0 0 1 18.25 21H5.75A2.75 2.75 0 0 1 3 18.25V5.75zM5.75 4.5c-.69 0-1.25.56-1.25 1.25v12.5c0 .69.56 1.25 1.25 1.25H6v-5.25A2.25 2.25 0 0 1 8.25 12h7.5A2.25 2.25 0 0 1 18 14.25v5.25h.25c.69 0 1.25-.56 1.25-1.25V8.286c0-.465-.184-.91-.513-1.238l-2.035-2.035a1.75 1.75 0 0 0-.952-.49V7.25a2.25 2.25 0 0 1-2.25 2.25h-4.5A2.25 2.25 0 0 1 7 7.25V4.5H5.75zm10.75 15v-5.25a.75.75 0 0 0-.75-.75h-7.5a.75.75 0 0 0-.75.75v5.25h9zm-8-15v2.75c0 .414.336.75.75.75h4.5a.75.75 0 0 0 .75-.75V4.5h-6z"
                    fill="currentColor"></path>
                </g>
              </svg>
            </template>
          </n-button>
          <n-button type="primary" @click="download" quaternary v-if="!hide">
            <!-- 下载epub -->
            <template #icon>
              <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 16 16">
                <g fill="none">
                  <path
                    d="M3.5 13h9a.75.75 0 0 1 .102 1.493l-.102.007h-9a.75.75 0 0 1-.102-1.493L3.5 13h9h-9zM7.898 1.007L8 1a.75.75 0 0 1 .743.648l.007.102v7.688l2.255-2.254a.75.75 0 0 1 .977-.072l.084.072a.75.75 0 0 1 .072.977l-.072.084L8.53 11.78a.75.75 0 0 1-.976.073l-.084-.073l-3.536-3.535a.75.75 0 0 1 .977-1.133l.084.072L7.25 9.44V1.75a.75.75 0 0 1 .648-.743L8 1l-.102.007z"
                    fill="currentColor"></path>
                </g>
              </svg>
            </template>
          </n-button>

          <n-button @click="prev" quaternary>
            <!-- 上一页 -->
            <template #icon>
              <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 20 20">
                <g fill="none">
                  <path
                    d="M6 4.75a.75.75 0 0 0-.743.648L5.25 5.5v9a.75.75 0 0 0 1.493.102l.007-.102v-9A.75.75 0 0 0 6 4.75zm8.28.22a.75.75 0 0 0-.976-.073l-.084.073l-4.5 4.5a.75.75 0 0 0-.073.976l.073.084l4.5 4.5a.75.75 0 0 0 1.133-.976l-.073-.084L10.31 10l3.97-3.97a.75.75 0 0 0 0-1.06z"
                    fill="currentColor"></path>
                </g>
              </svg>
            </template>
          </n-button>
          <n-button @click="next" quaternary>
            <!-- 下一页 -->
            <template #icon>
              <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 20 20">
                <g fill="none">
                  <path
                    d="M13.75 4.75a.75.75 0 0 1 .743.648l.007.102v9a.75.75 0 0 1-1.493.102L13 14.5v-9a.75.75 0 0 1 .75-.75zm-8.28.22a.75.75 0 0 1 .976-.073l.084.073l4.5 4.5a.75.75 0 0 1 .073.976l-.073.084l-4.5 4.5a.75.75 0 0 1-1.133-.976l.073-.084L9.44 10L5.47 6.03a.75.75 0 0 1 0-1.06z"
                    fill="currentColor"></path>
                </g>
              </svg> </template>
          </n-button>



        </div>
      </n-spin>
      <div v-if="content != ''" class="text">
        <textarea v-model="content" v-if="!isPreview" ref="text"></textarea>
        <iframe v-else class="preview" ref="preview"></iframe>
      </div>
    </div>


  </div>
</template>
<style lang="less">
.body {
  & {
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

  .expand-width {
    width: 200px;
  }

  .body {
    height: 100%;
  }

  .hide {
    margin-left: 200px;
  }

  .right-content {
    height: 100%;
  }

  .right-content .text {
    margin-top: -70px;
    padding-top: 70px;
    height: 100%;
    width: 100%;
    /* padding-bottom: 40px; */
    padding-left: 20px;
    padding-right: 40px;
  }

  .n-upload {
    display: inline !important;
  }

  .right-content .text>textarea {
    width: 100%;
    padding-left: 5px;
    padding-right: 5px;
    resize: none;
    height: 100%;
    background-color: inherit;
    color: inherit;
    font-size: 18px;
  }

  .right-content .text .preview {
    width: 100%;
    height: 100%;
    margin: 0px;
  }
}
</style>
