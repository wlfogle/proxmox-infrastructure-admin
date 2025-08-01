<template>
  <div class="containers">
    <h1>Containers</h1>
    <div v-if="loading">Loading...</div>
    <div v-else>
      <table>
        <thead>
          <tr>
            <th>Name</th>
            <th>Status</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="container in containers" :key="container.id">
            <td>{{ container.name }}</td>
            <td>{{ container.status }}</td>
            <td>
              <button @click="startContainer(container.id)" :disabled="container.status === 'running'">Start</button>
              <button @click="stopContainer(container.id)" :disabled="container.status !== 'running'">Stop</button>
              <button @click="restartContainer(container.id)">Restart</button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export default {
  name: 'Containers',
  setup() {
    const containers = ref([])
    const loading = ref(false)

    const fetchContainers = async () => {
      try {
        loading.value = true
        const result = await invoke('get_system_overview')
        containers.value = result.containers
      } catch (error) {
        console.error('Failed to fetch containers:', error)
      } finally {
        loading.value = false
      }
    }

    const startContainer = async (id) => {
      try {
        await invoke('start_container', { container_id: id })
        await fetchContainers()
      } catch (error) {
        console.error('Failed to start container:', error)
      }
    }

    const stopContainer = async (id) => {
      try {
        await invoke('stop_container', { container_id: id })
        await fetchContainers()
      } catch (error) {
        console.error('Failed to stop container:', error)
      }
    }

    const restartContainer = async (id) => {
      try {
        await invoke('restart_container', { container_id: id })
        await fetchContainers()
      } catch (error) {
        console.error('Failed to restart container:', error)
      }
    }

    onMounted(fetchContainers)

    return {
      containers,
      loading,
      startContainer,
      stopContainer,
      restartContainer
    }
  }
}
</script>

<style scoped>
.containers {
  padding: 20px;
}

button {
  margin-right: 5px;
}
table {
  width: 100%;
  border-collapse: collapse;
  margin-top: 20px;
}
th, td {
  border: 1px solid #ddd;
  padding: 8px;
}
th {
  background-color: #f2f2f2;
}
</style>

