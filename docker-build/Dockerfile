FROM ubuntu:22.04
ENV LANG=C.UTF-8 TZ=Asia/Shanghai

RUN sed -i "s@http://.*archive.ubuntu.com@http://mirrors.huaweicloud.com@g" /etc/apt/sources.list \
    && sed -i "s@http://.*security.ubuntu.com@http://mirrors.huaweicloud.com@g" /etc/apt/sources.list

RUN mkdir -p /apps /data /public
VOLUME /data /public
WORKDIR /apps

COPY sse-queue /usr/bin/sse-queue
COPY config.toml /apps/config.toml

EXPOSE 8585
CMD ["sse-queue","-c","/apps/config.toml"]