# 多阶段构建
FROM rust:1.75 AS builder

WORKDIR /app

# 复制依赖文件
COPY Cargo.toml ./

# 复制源代码
COPY src ./src

# 构建发布版本
RUN cargo build --release

# 运行阶段
FROM debian:bookworm-slim

# 安装 CA 证书和 SSL 库
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/resume-insight /app/

# 暴露端口
EXPOSE 3000

# 运行服务
CMD ["./resume-insight"]
