FROM {{ manifest.binary.os }}

RUN apt-get -qq update && apt-get -yqq upgrade && apt-get install -yqq socat

LABEL maintainer="soma_{{ username }}"
LABEL soma.username="{{ username }}"
LABEL soma.version="{{ version }}"
LABEL soma.repository="{{ repo_name }}"
LABEL soma.problem="{{ manifest.name }}"

{{ #with manifest }}
ENV PROB "{{ name }}"
RUN useradd -m $PROB

{{ #each binary.file_entries }}
COPY "{{ path }}" "{{ target_path }}"
RUN chmod {{ permissions }} "{{ target_path }}"
{{ /each }}

RUN chown -R root:$PROB /home/$PROB

COPY ./start.sh /start.sh
RUN chmod 555 /start.sh

USER $PROB
WORKDIR /home/$PROB
CMD ["/start.sh"]

# TODO: Container internal port settings may be implemented afterwards
EXPOSE 1337
{{ /with }}

#RUN apt install -y tzdata
#ENV TZ=Asia/Seoul
#RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone
