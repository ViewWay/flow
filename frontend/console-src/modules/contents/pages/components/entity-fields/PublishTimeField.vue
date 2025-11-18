<script lang="ts" setup>
import type { ListedSinglePage } from "@flow-dev/api-client";
import { VEntityField } from "@flow-dev/components";
import { utils } from "@flow-dev/console-shared";

withDefaults(
  defineProps<{
    singlePage: ListedSinglePage;
  }>(),
  {}
);
</script>

<template>
  <VEntityField
    v-if="singlePage.page.spec.publishTime"
    v-tooltip="utils.date.format(singlePage.page.spec.publishTime)"
    :description="utils.date.timeAgo(singlePage.page.spec.publishTime)"
  >
  </VEntityField>
</template>
