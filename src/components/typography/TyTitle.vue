<script setup lang="ts">
defineProps<{
  value?: string;
}>();
</script>

<template>
  <div class="text-h5">
    <slot>
      {{ value }}
    </slot>
  </div>
</template>
