FROM node:18.18.0 as html
COPY ./html /app/
RUN cd /app/ && npm --registry https://registry.npmmirror.com/ i && npm run build

FROM inkbox/rust:1.75.0 as rust
WORKDIR /workdir
RUN cargo new app && apt install -y musl-tools && rustup target add $(arch)-unknown-linux-musl
COPY ./rust/Cargo.toml /workdir/app
# 安装依赖
RUN cd /workdir/app && RUSTC_LINKER=musl-gcc cargo build --release --target=$(arch)-unknown-linux-musl
COPY ./rust/ /workdir/app/
# 拷贝前端代码
COPY --from=html /app/dist /workdir/app/static
# 安装项目
RUN cd /workdir/app && tree && rm -rf /workdir/app/target/$(arch)-unknown-linux-musl/release/deps/zip_server-*  \
  && RUSTC_LINKER=musl-gcc cargo build --release -vv --target=$(arch)-unknown-linux-musl\
  && strip -s /workdir/app/target/$(arch)-unknown-linux-musl/release/zip-server  \
  &&  ls -all -h /workdir/app/target/$(arch)-unknown-linux-musl/release/zip-server && cp /workdir/app/target/$(arch)-unknown-linux-musl/release/zip-server /zip-server 


FROM busybox
# scratch 会遇到 日志的 时区问题,可以解决,但是懒得弄了
# COPY --from=rust /etc/ssl /etc/ssl
# COPY --from=0 /usr/share/ca-certificates /usr/share/ca-certificates
COPY --from=rust /zip-server /server
CMD ["/server"]
