<script setup lang="ts">
/** 落地站根组件：按序渲染全部区块 */
import SiteNav from "./components/SiteNav.vue";
import Hero from "./components/Hero.vue";
import Features from "./components/Features.vue";
import AppMock from "./components/AppMock.vue";
import Download from "./components/Download.vue";
import Changelog from "./components/Changelog.vue";
import Resources from "./components/Resources.vue";
import Faq from "./components/Faq.vue";
import SiteFooter from "./components/SiteFooter.vue";
</script>

<template>
  <SiteNav />
  <main>
    <Hero />
    <Features />
    <AppMock />
    <Download />
    <Changelog />
    <Resources />
    <Faq />
  </main>
  <SiteFooter />
</template>
