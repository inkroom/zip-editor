<script>

export default{

    methods:{
        onPrev(){
            this.$emit('prev');
        },
        onNext(){
            console.log('next')

            this.$emit('next');
        }
    }
}

</script>
<style>
.page-button{
    /* height: 20px; */

    svg{
        height: 25px;
        width: 25px;
    }
}
</style>
<template>
    <div class="page-button">
        <!-- 上一页 -->
        <div @click="onPrev">
            <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 20 20">
                <g fill="none">
                    <path
                        d="M6 4.75a.75.75 0 0 0-.743.648L5.25 5.5v9a.75.75 0 0 0 1.493.102l.007-.102v-9A.75.75 0 0 0 6 4.75zm8.28.22a.75.75 0 0 0-.976-.073l-.084.073l-4.5 4.5a.75.75 0 0 0-.073.976l.073.084l4.5 4.5a.75.75 0 0 0 1.133-.976l-.073-.084L10.31 10l3.97-3.97a.75.75 0 0 0 0-1.06z"
                        fill="currentColor"></path>
                </g>
            </svg>
        </div>
        <!-- 下一页 -->
        <div @click="onNext">
            <svg @click="next" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 20 20">
                <g fill="none">
                    <path
                        d="M13.75 4.75a.75.75 0 0 1 .743.648l.007.102v9a.75.75 0 0 1-1.493.102L13 14.5v-9a.75.75 0 0 1 .75-.75zm-8.28.22a.75.75 0 0 1 .976-.073l.084.073l4.5 4.5a.75.75 0 0 1 .073.976l-.073.084l-4.5 4.5a.75.75 0 0 1-1.133-.976l.073-.084L9.44 10L5.47 6.03a.75.75 0 0 1 0-1.06z"
                        fill="currentColor"></path>
                </g>
            </svg>
        </div>
    </div>
</template>