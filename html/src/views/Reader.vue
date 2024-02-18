<script>
import axios from 'axios';
import { NButton, NUpload, NTree, useMessage, NSpin } from 'naive-ui';
import { h } from 'vue';

import localForage from 'localforage';
import PageButton from './PageButton.vue';
import jszip from 'jszip';

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
  var r = location.pathname.replace('/reader', '');
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
  components: { NButton, NUpload, NTree, NSpin, PageButton },
  data() {

    let _this = this;
    return {
      zip: null,
      message: useMessage(),
      data: [],
      loading: false,
      originData: [],
      content: '',
      path: '',
      parent_id: null,
      file: this.$route.params.file,
      hasFile: this.$route.params.file != undefined,
      isPreview: true,
      previewContent: '',
      hide: false,
      expandedKeys: ['expand'],
      nodeProps: ({ option }) => {
        return {
          onClick() {
            // let data = option;
            // if (data.children) {
            //   if (_this.lastExpandData && _this.lastExpandData != data && _this.lastExpandData.level == data.level) {
            //     // 展开互斥，一次只能展开一个节点
            //     _this.lastExpandData.expand = false;
            //   }
            //   _this.lastExpandData = data;
            // }
            // if (!option.children) {
            //   option.expand = true;
            // }
            _this.handleTreeClick(option)
          }
        }
      }
    }

  },
  methods: {

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
        console.log('记录 ', this.path);
        previewScrollTop[this.file + '/' + this.path] = iframedoc.documentElement.scrollTop;
      }

      this.zip.file(current.path).async('string')
        .then(res => {
          this.content = res;
          this.path = current.path;
          this.parent_id = current.parent_id;

          let s = localStorage.getItem('progress');
          if (s == null || s == '') {
            let m = {};
            m[this.file] = this.path;
            s = JSON.stringify(m);
            localStorage.setItem('progress', s);
          } else {
            s = JSON.parse(s);
            s[this.file] = this.path;
            localStorage.setItem('progress', JSON.stringify(s));
          }

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
        className: (data.path == this.path || this.parent_id == (data.id)) ? 'highlight' : '',
        style: {
          display: 'inline-block',
          float: 'right',
          // fontWeight: (data.path == this.path || this.parent_id.startsWith(data.id)) ? '800' : 400
        },
        innerHTML: data.title,
      });
    },

    getPreviewContent() {
      if (this.isPreview) {
        // 处理外部依赖
        // let p = document.createElement('div');
        let p = new DOMParser().parseFromString(this.content, 'text/html');

        let promises = [];

        this.loading = true;
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
          img.setAttribute('style', "width:100%");

          // 图片转 base64
          console.log('new_src', new_src)
          promises.push(this.zip.file(new_src).async('base64').then((base64) => {
            img.setAttribute('src', `data:image/${new_src.substring(new_src.lastIndexOf('.') + 1)};base64,` + base64);
          }))


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

          // link 直接获取内容写入 文本


          promises.push(this.zip.file(new_src).async('string').then(css => {
            let sty = p.createElement('style');
            sty.innerHTML = css;
            let h = p.getElementsByTagName('head')[0];
            h.appendChild(sty);
            h.removeChild(link);
          }))
          // link.setAttribute('href', location.protocol + "//" + location.host + "/api/assets/" + this.file + "?path=" + encodeURIComponent(new_src));

        }
        Promise.all(promises).then(() => {
          this.loading = false;
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
              console.log('previewScrollTop', previewScrollTop, this.path);
              if (previewScrollTop[this.file + "/" + this.path])
                iframedoc.documentElement.scrollTop = previewScrollTop[this.file + "/" + this.path];
              else {
                iframedoc.documentElement.scrollTop = 0;
              }
              iframedoc.documentElement.style = 'color: #afb5c2'
            })
          })
        })


      }
    },

    eclpise() {

      this.hide = !this.hide;

    },
    find_in_tree(tree, path) {
      if (Array.isArray(tree)) {
        for (let i = 0; i < tree.length; i++) {
          let s = this.find_in_tree(tree[i], path);
          if (s != null) {
            return s;
          }
        }
        return null;
      }

      if (tree.path == path) return tree;
      if (tree.children) {
        for (let i = 0; i < tree.children.length; i++) {
          let s = this.find_in_tree(tree.children[i], path);
          if (s != null) {
            return s;
          }
        }
      }
      return null;
    },
    seek(sep) {
      let f = this.path;
      let index = this.originData.findIndex(s => s.name == f);
      if (index != -1) {
        index += sep;
        if (index == this.originData.length) return;
        if (index == -1) return;
        let new_path = this.originData[index].name;

        // 找到树节点

        let s = this.find_in_tree(this.data, new_path)
        if (s == null) return;

        this.handleTreeClick(s);
      }

    },
    prev() {
      // 上一个
      this.seek(-1);
    },
    next() {
      console.log('next  33')
      this.seek(1);
    }
    , loadZip() {
      this.loading = true;
      document.title = decodeURIComponent(this.file.substring(this.file.lastIndexOf("/")+1))
      localForage.getItem('epub/' + this.file)
        .then(value => value == null ? Promise.reject(value) : value)
        .catch(e => axios.get(`${this.file}`, { responseType: "blob" }).then(res => res.data))
        .then(value => { localForage.setItem('epub/' + this.file, value); return value; })

        // axios.get(`${this.file}`, { responseType: "blob" })
        .then(res => jszip.loadAsync(res))
        .then(zip => this.zip = zip)
        .then(zip => zip.file('mimetype').async('string'))
        .then(out => {
          console.log('out', out);
        })
        // 获取目录
        .then(() => this.zip.file('EPUB/toc.ncx').async('string'))
        .then(toc => {

          // 解析目录
          let p = new DOMParser().parseFromString(toc, 'text/xml');

          let c = p.children[0].children;
          for (let i = 0; i < c.length; i++) {
            if (c[i].nodeName == 'navMap') {

              // 底下可能有多级 navPoint，涉及目录嵌套
              return c[i].children;
            }
          }
          return [];
        })
        .then(navPoint => {

          let nav = [];

          function loadNav(node, level) {
            let text = '';
            let src = '';
            for (let i = 0; i < node.children.length; i++) {
              let s = node.children[i];
              if (s.nodeName == 'navLabel') {
                text = s.getElementsByTagName('text')[0].innerHTML;
              } else if (s.nodeName == 'content') {
                src = s.getAttribute('src');
                if (src[0] != '/') {
                  src = 'EPUB/' + src;
                }
              }
            }

            return { title: text, path: src, level, id: node.id };
          }

          for (let i = 0; i < navPoint.length; i++) {
            // 判断是否有嵌套，暂时只考虑嵌套两层的
            console.log('c', navPoint[i].children)
            let next = Array.from(navPoint[i].children).findIndex(s => s.nodeName == 'navPoint');
            if (next != -1) {
              // 还有嵌套
              let n = loadNav(navPoint[i], 1);
              n.children = [];
              let children = Array.from(navPoint[i].getElementsByTagName('navPoint'));
              children.forEach(s => {
                let c = loadNav(s, 2)
                c.parent_id = n.id;// 记录上级
                n.children.push(c);
              });

              nav.push(n);
            } else {
              nav.push(loadNav(navPoint[i], 1));
            }

          }

          // 转换 nav ，主要是拉平，只要叶子节点，用于翻页
          let org = [];
          nav.forEach(n => {
            if (n.children) {
              n.children.forEach(c => {
                org.push({ name: c.path });
              })

            } else {
              org.push({ name: n.path });
            }
          })
          this.originData = org;
          this.data = nav;
          console.log('nav', nav);
        }).then(_ => {
          // 加载进度
          let s = localStorage.getItem('progress');
          if (s != null && s != '') {
            s = JSON.parse(s);
            if (s[this.file]) {

              console.log('new_path', s[this.file])
              let m = this.find_in_tree(this.data, s[this.file])
              if (m == null) return;

              this.handleTreeClick(m);
            }



          }

        })
        .catch(e => {
          this.message.error('加载epub失败');
          console.error(e);
        })
        .finally(_ => this.loading = false)

    }
    , refresh() {
      localForage.removeItem('epub/' + this.file).then(this.loadZip);
    }
  }
  , mounted() {
    this.loadZip();


  }
}
</script>

<template>
  <div class="reader-body">
    <PageButton class="left-button" v-if="hide" @prev="prev" @next="next">

    </PageButton>
    <PageButton class="right-button" v-if="hide" @prev="prev" @next="next">

    </PageButton>
    <div class="left tree" v-if="!hide">

      <n-tree :data="data" v-if="hasFile" @on-select-change="handleTreeClick" :expand-on-click="true"
        :node-props="nodeProps" key-field="path" label-field="title" @on-toggle-expand="handleTreeClick"
        :render-label="renderTreeContent"></n-tree>

    </div>

    <div :class="hide ? 'right-content' : 'right-content hide'">
      <n-spin :show="loading">
        <div :class="hide ? 'header hide-header' : 'header'">
          <div>
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
          </div>
          <div>
            <n-button type="primary" @click="refresh" quaternary>
              <!-- 删除缓存 -->
              <template #icon>
                <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 512 512">
                  <path d="M320 146s24.36-12-64-12a160 160 0 1 0 160 160" fill="none" stroke="currentColor"
                    stroke-linecap="square" stroke-miterlimit="10" stroke-width="32"></path>
                  <path fill="none" stroke="currentColor" stroke-linecap="square" stroke-miterlimit="10" stroke-width="32"
                    d="M256 58l80 80l-80 80"></path>
                </svg>
              </template>
            </n-button>
          </div>
          <div>
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
          </div>
          <div>
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

        </div>
      </n-spin>

      <div v-if="content != ''" class="text">

        <iframe class="preview" ref="preview"></iframe>
      </div>
    </div>


  </div>
</template>
<style lang="less">
.reader-body {
  & {
    height: 100%;
  }

  .content {
    height: 100%;
  }

  .header {
    display: table;
    width: 100%;

    &>div {
      display: table-cell;
      text-align: center;
    }
  }

  .hide-header {
    button {
      width: 25%;
      margin-left: 0px !important;
      margin-right: 0px !important;
    }
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


  .hide {
    margin-left: 200px;
  }

  .right-content {
    height: 100%;

    .text {
      margin-top: -70px;
      padding-top: 70px;
      height: 100%;
      width: 100%;
      /* padding-bottom: 40px; */
      padding-left: 20px;
      padding-right: 20px;

      .preview {
        width: 100%;
        height: 100%;
        margin: 0px;
      }
    }
  }





  .left-button {
    position: fixed;
    top: 50%;

    button {
      display: block;
      margin-left: 0px !important;
      padding-left: 0px !important;
      width: 40px;
      height: 40px;
    }
  }

  .right-button {
    position: fixed;
    top: 50%;
    right: 8px;

    button {
      display: block;
      margin-right: 0px !important;
      padding-right: 0px !important;
      width: 40px;
      height: 40px;
    }
  }
}
</style>