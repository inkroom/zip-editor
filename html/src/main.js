import './assets/main.css'
import ViewUIPlus from 'view-ui-plus'
import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import 'view-ui-plus/dist/styles/viewuiplus.css'
const app = createApp(App)

app.use(router).use(ViewUIPlus)

app.mount('#app')
