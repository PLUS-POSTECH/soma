FROM {{ binary.os }}

RUN apt-get -qq update && apt-get -yqq upgrade && apt-get install -yqq socat

COPY image-root/ /

ENV PROB "{{ name }}"
RUN useradd -m $PROB

COPY .soma/ /.soma

RUN chmod 555 /.soma/configure_permissions.sh \
    && /.soma/configure_permissions.sh \
    && rm /.soma/configure_permissions.sh

USER $PROB
WORKDIR {{ work_dir }}
CMD ["/.soma/start.sh"]

# TODO: Container internal port settings may be implemented afterwards
EXPOSE 1337

#RUN apt install -y tzdata
#ENV TZ=Asia/Seoul
#RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone
