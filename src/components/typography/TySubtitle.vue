<script setup lang="ts">
defineProps<{
  value?: string;
}>();
</script>

<template>
  <div class="text-subtitle1">
    <slot>
      {{ value }}
    </slot>
  </div>
</template>
