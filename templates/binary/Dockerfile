FROM {{ manifest.binary.os }}

LABEL maintainer="soma_{{ username }}"
LABEL soma.username="{{ username }}"
LABEL soma.version="{{ version }}"
LABEL soma.repository="{{ repository_name }}"

RUN apt update && apt -y upgrade
RUN apt install -y socat

{{ #with manifest }}
ENV PROB "{{ name }}"
RUN useradd -m $PROB
RUN chown -R root:$PROB /home/$PROB

ADD deploy/ /

{{ #each executable }}
RUN chmod +x "{{ this.path }}"
{{ /each }}

{{ #each readonly }}
RUN chmod -w "{{ this.path }}"
{{ /each }}

COPY ./start.sh /start.sh
RUN chmod 555 /start.sh

USER $PROB
CMD ["/start.sh"]

# TODO: Container internal port settings may be implemented afterwards
EXPOSE 1024
{{ /with }}

#RUN apt install -y tzdata
#ENV TZ=Asia/Seoul
#RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone
