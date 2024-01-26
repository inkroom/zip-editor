package cn.inkroom.zip;

import net.lingala.zip4j.model.ZipParameters;
import org.junit.jupiter.api.Test;

import java.lang.reflect.Method;

//@SpringBootTest
class ZipApplicationTests {

    @Test
    void contextLoads() {

        Method[] methods = ZipParameters.class.getMethods();
        for (Method method : methods) {
            String name = method.getName();
            if (name.startsWith("set"))
                System.out.println("zipParameters." + name + "( fileHeader.get" + name.substring(3) + "() );");
        }

    }

}
