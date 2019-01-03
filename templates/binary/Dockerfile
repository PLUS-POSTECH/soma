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

{{ #each executable }}
COPY "{{ this.path }}" "{{ this.target_path }}"
RUN chmod +x "{{ this.target_path }}"
{{ /each }}

{{ #each readonly }}
COPY "{{ this.path }}" "{{ this.target_path }}"
RUN chmod -w "{{ this.target_path }}"
{{ /each }}

RUN chown -R root:$PROB /home/$PROB

COPY ./start.sh /start.sh
RUN chmod 555 /start.sh

USER $PROB
WORKDIR /home/$PROB
CMD ["/start.sh"]

# TODO: Container internal port settings may be implemented afterwards
EXPOSE 1024
{{ /with }}

#RUN apt install -y tzdata
#ENV TZ=Asia/Seoul
#RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone
