# zip-editor

web版的zip文件编辑器,只支持修改文本文件内容,不支持二进制文件,不支持添加删除文件等


主要用于修改 epub 文件,因为epub也是zip格式的

# 结构

- server java后端
- html vue3前端

# 构建

参见 [Dockerfile](Dockerfile)



# 运行

docker run -itd -v zip:/zip -p 3992:3992 -e PORT=3992 -e FILE_SIZE=50M ghcr.io/inkroom/zip-editor 
