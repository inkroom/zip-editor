package cn.inkroom.zip;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.boot.web.server.ConfigurableWebServerFactory;
import org.springframework.boot.web.server.ErrorPage;
import org.springframework.boot.web.server.WebServerFactoryCustomizer;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.http.HttpStatus;

@Configuration
@SpringBootApplication
public class ZipApplication {

    @Bean
    public WebServerFactoryCustomizer<ConfigurableWebServerFactory> s() {
        return factory -> factory.addErrorPages(new ErrorPage(HttpStatus.NOT_FOUND, "/index.html"));
    }

    public static void main(String[] args) {
        SpringApplication.run(ZipApplication.class, args);
    }

}
