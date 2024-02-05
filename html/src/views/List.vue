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
      previewContent: '',
      lastExpandData: undefined
    }

  },
  mounted(){
    axios.get('/api/dir_list').then(res=>{
                  // 组装成树
          let tree = [];

             // 返回的数据是一级一级的，所以重新添加
            let data = [];
            for(let i = res.data.length -1 ;i>=0 ;i--){
              if(!data.some(s=>s.name.startsWith(res.data[i].name))){
                data.push(res.data[i]);
              }
            }

          data.forEach(m => {

         


            // 拆分目录结构
            let dir = m.name.split('/');
            this._find_parent(tree, dir, m.name, "")



          });

          this.data = tree;
    }).catch(error => {
          console.error('文件list失败', error);
          this.$Message.error('文件list失败');

        });
  },
  methods: {
    _find_parent(tree, dir, path, parent, level) {
      if (!level) level = 1;
      if (dir.length == 1) {// 最后一级，暂时认为只能是文件
        tree.push({ title: dir[0], expand: false, path, level });
        return;
      }

      for (var i = 0; i < tree.length; i++) {
        if (tree[i].title == dir[0]) {
          dir.shift();// 删除第一个元素

          this._find_parent(tree[i].children, dir, path, tree[i].path + "/", level + 1);
          return;
        }

      }
      // 找完了都没有，就创建一层
      var new_tree = { title: dir[0], children: [], expand: false, path: parent + dir[0], level };
      tree.push(new_tree);
      // 继续往下
      dir.shift();
      this._find_parent(new_tree.children, dir, path, parent + dir[0] + '/', level + 1);


    },

    handleTreeClick(all, current) {
      if (!current) {
        current = all;
      }
      if (!current.path) {
        return;
      }
      if (current.children) {
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
      window.open(location.protocol+'//'+location.host+'/'+ encodeURIComponent(current.path) )
    },
    renderTreeContent(h, { root, node, data }) {
      return h('span', {
        style: {
          display: 'inline-block',
          float: 'right',
          marginRight: '32px',
          fontWeight: (data.path == this.path || this.path.startsWith(data.path)) ? '800' : 400
        },
        innerHTML: data.title,
        onClick: () => {
          if (data.children) {
            if (this.lastExpandData && this.lastExpandData != data && this.lastExpandData.level == data.level) {
              // 展开互斥，一次只能展开一个节点
              this.lastExpandData.expand = false;
            }
            this.lastExpandData = data;
          }
          this.handleTreeClick(data)
        }
      });
    },


  }
}
</script>

<template>

    <div class="list">
      <Tree :data="data" @on-select-change="handleTreeClick" :expand-node="true"
        @on-toggle-expand="handleTreeClick" :render="renderTreeContent"></Tree>

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
}
</style>