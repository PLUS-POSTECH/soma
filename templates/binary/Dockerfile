FROM {{ os }}

LABEL maintainer="soma_{{ username }}"
LABEL soma.id="{{ soma.id }}"
LABEL soma.version="{{ soma.version }}"
LABEL soma.repository="{{ repository }}"

RUN apt update && apt -y upgrade
RUN apt install -y socat

ENV PROB "{{ name }}"
RUN useradd -m $PROB
RUN chown -R root:$PROB /home/$PROB

ADD deploy/ /

{{ #each executable }}
RUN chmod +x "{{ path }}"
{{ /each }}

{{ #each readonly }}
RUN chmod -w "{{ path }}"
{{ /each }}

COPY ./start.sh /start.sh
RUN chmod 555 /start.sh

USER $PROB
CMD ["/start.sh"]

EXPOSE {{ port }}

#RUN apt install -y tzdata
#ENV TZ=Asia/Seoul
#RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone
