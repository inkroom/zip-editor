import './assets/main.less'
import { createApp } from 'vue'
import App from './App.vue'
import router from './router'

import localForage from 'localforage';

localForage.config({driver:localForage.INDEXEDDB,name:'reader',storeName:'reader'})

const app = createApp(App)

app.use(router)

app.mount('#app')
