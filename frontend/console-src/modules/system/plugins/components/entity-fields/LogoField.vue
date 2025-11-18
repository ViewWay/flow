<script lang="ts" setup>
import type { Plugin } from "@flow-dev/api-client";
import { VAvatar, VEntityField } from "@flow-dev/components";
withDefaults(
  defineProps<{
    plugin: Plugin;
  }>(),
  {}
);
</script>

<template>
  <VEntityField>
    <template #description>
      <VAvatar
        :alt="plugin.spec.displayName"
        :src="plugin.status?.logo"
        size="md"
      ></VAvatar>
    </template>
  </VEntityField>
</template>
