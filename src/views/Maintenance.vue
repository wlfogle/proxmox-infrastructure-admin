<template>
  <div class="maintenance">
    <h1>Maintenance</h1>
    <div v-if="loading">Loading...</div>
    <div v-else>
      <div class="maintenance-section">
        <h2>Services</h2>
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>Status</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="service in maintenance.services" :key="service.name">
              <td>{{ service.name }}</td>
              <td>{{ service.status }}</td>
              <td>
                <button @click="controlService(service.name, 'start')" :disabled="service.active">Start</button>
                <button @click="controlService(service.name, 'stop')" :disabled="!service.active">Stop</button>
                <button @click="controlService(service.name, 'restart')">Restart</button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <div class="maintenance-section">
        <h2>Binaries</h2>
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>Path</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="binary in maintenance.binaries" :key="binary.name">
              <td>{{ binary.name }}</td>
              <td>{{ binary.path }}</td>
            </tr>
          </tbody>
        </table>
      </div>
      <div class="maintenance-section">
        <h2>Configs</h2>
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>Path</th>
              <th>Size (KB)</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="config in maintenance.configs" :key="config.name">
              <td>{{ config.name }}</td>
              <td>{{ config.path }}</td>
              <td>{{ (config.size / 1024).toFixed(2) }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export default {
  name: 'Maintenance',
  setup() {
    const maintenance = ref({ services: [], binaries: [], configs: [] })
    const loading = ref(false)

    const fetchMaintenanceOverview = async () => {
      try {
        loading.value = true
        const result = await invoke('get_maintenance_overview')
        maintenance.value = result
      } catch (error) {
        console.error('Failed to fetch maintenance overview:', error)
      } finally {
        loading.value = false
      }
    }

    const controlService = async (name, action) => {
      try {
        await invoke('control_service', { service_name: name, action })
        await fetchMaintenanceOverview()
      } catch (error) {
        console.error(`Failed to ${action} service:`, error)
      }
    }

    onMounted(fetchMaintenanceOverview)

    return {
      maintenance,
      loading,
      controlService
    }
  }
}
</script>

<style scoped>
.maintenance {
  padding: 20px;
}

button {
  margin-right: 5px;
}

.maintenance-section {
  margin-top: 20px;
}

table {
  width: 100%;
  border-collapse: collapse;
  margin-top: 10px;
}

th, td {
  border: 1px solid #ddd;
  padding: 8px;
}

th {
  background-color: #f2f2f2;
}
</style>

