<script>
import axios from 'axios';
import { h } from 'vue';
import { NTree } from 'naive-ui';


export default {
  components: { NTree },
  data() {
    let _this = this;
    return {
      data: [],
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
  mounted() {
    axios.get('/api/dir_list').then(res => {
      // 组装成树
      let tree = [];

      // 返回的数据是一级一级的，所以重新添加
      let data = [];
      for (let i = res.data.length - 1; i >= 0; i--) {
        if (!data.some(s => s.name.startsWith(res.data[i].name))) {
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

      window.open(location.protocol + '//' + location.host + '/' + encodeURIComponent(current.path))
    },
    renderTreeContent({ option }) {
      let data = option;
      return h('span', {
        style: {
          display: 'inline-block',
          float: 'right',
          marginRight: '32px',
        },
        innerHTML: data.title,
      });
    },


  }
}
</script>

<template>
  <div class="list">
    <n-tree :data="data" @on-select-change="handleTreeClick" :expand-node="true" :node-props="nodeProps"
      :expand-on-click="true" key-field="path" label-field="title" @on-toggle-expand="handleTreeClick"
      :render-label="renderTreeContent"></n-tree>

  </div>
</template>
<style></style>
