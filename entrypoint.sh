
PORT=${PORT-38231}
FILE_SIZE=${FILE_SIZE-50M}
java -jar /server.jar --server.port=${PORT} --spring.servlet.multipart.max-file-size=${FILE_SIZE} --spring.servlet.multipart.max-request-size=${FILE_SIZE}

