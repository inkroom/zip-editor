FROM node:18.18.0 as html
COPY ./html /app/
RUN cd /app/ && npm --registry https://registry.npmmirror.com/ i && npm run build 

FROM eclipse-temurin:21.0.2_13-jdk as server

ARG MAVEN_HOME=/usr/local/lib/maven
ARG MAVEN_VERSION=3.9.6

ENV PATH ${PATH}:${MAVEN_HOME}/apache-maven-${MAVEN_VERSION}/bin

#RUN wget -q https://repo.huaweicloud.com/apache/maven/maven-3/${MAVEN_VERSION}/binaries/apache-maven-${MAVEN_VERSION}-bin.tar.gz && mkdir -p ${MAVEN_HOME} && tar -zxvf apache-maven-${MAVEN_VERSION}-bin.tar.gz -C ${MAVEN_HOME} && rm -rf apache-maven-${MAVEN_VERSION}-bin.tar.gz \
#  && ( cat ${MAVEN_HOME}/apache-maven-${MAVEN_VERSION}/conf/settings.xml | sed  "55i<localRepository>/project/maven/</localRepository>" | sed "161i</mirror>" | sed "161i    <url>https://repo.huaweicloud.com/repository/maven/</url>" | sed "161i    <mirrorOf>*</mirrorOf>" | sed "161i    <id>huaweicloud</id>" | sed  "161i<mirror>" | sed  "166i<\!--" | sed  "174i-->" > ${MAVEN_HOME}/apache-maven-${MAVEN_VERSION}/conf/settings.xml  ) \
 # && java -version && javac --version && mvn --version

RUN wget -q https://repo.huaweicloud.com/apache/maven/maven-3/${MAVEN_VERSION}/binaries/apache-maven-${MAVEN_VERSION}-bin.tar.gz && mkdir -p ${MAVEN_HOME} && tar -zxvf apache-maven-${MAVEN_VERSION}-bin.tar.gz -C ${MAVEN_HOME} && rm -rf apache-maven-${MAVEN_VERSION}-bin.tar.gz   && java -version && javac --version && mvn --version




COPY ./server /app/
COPY --from=html /app/dist /app/src/main/resources/static

RUN  --mount=type=cache,mode=0777,target=/root/.m2/ cd /app/ && mvn clean package -Dmaven.test.skip=true -Dmaven.repo.local=/root/.m2


FROM eclipse-temurin:21-jre-alpine
COPY --from=server /app/target/server.jar /server.jar
COPY ./entrypoint.sh /entrypoint.sh

VOLUME /zip
ENTRYPOINT ["sh","/entrypoint.sh"]


