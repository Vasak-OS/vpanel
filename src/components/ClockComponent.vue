<script lang="ts" setup>
import { ref } from 'vue';

const day = ref('00');
const month = ref('00');
const year = ref('0000');
const min = ref('00');
const hour = ref('00');

setInterval(() => {
  const date = new Date();
  hour.value = dobleDTO(date.getHours()); // 0 - 23
  min.value = dobleDTO(date.getMinutes()); // 0 - 59
  day.value = dobleDTO(date.getDate());
  month.value = dobleDTO(date.getMonth() + 1);
  year.value = date.getFullYear().toString();
}, 5000);

function dobleDTO(data: number): string {
  return data < 10 ? `0${data}` : data.toString();
}
</script>

<template>
  <div id="clock">
    <span
      data-bs-toggle="tooltip"
      data-bs-placement="left"
      :title="
        day.toString().padStart(2, '0') + '/' + month.toString().padStart(2, '0') + '/' + year
      "
    >
      {{ hour }}:{{ min }}
    </span>
  </div>
</template>
