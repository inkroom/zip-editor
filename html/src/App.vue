<script >
import { RouterLink, RouterView } from 'vue-router'
import { darkTheme, NConfigProvider, NMessageProvider } from 'naive-ui';

export default {
  components: { NConfigProvider, NMessageProvider },
  data() {
    return {
      darkTheme
    }
  }
}

</script>

<template>
  <n-message-provider>
    <n-config-provider :theme="darkTheme">

      <RouterView />
    </n-config-provider>
  </n-message-provider>
</template>
