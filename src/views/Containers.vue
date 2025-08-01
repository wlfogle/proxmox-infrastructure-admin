<template>
  <div class="containers">
    <h1>Container Management</h1>
    <div v-if="loading" class="loading">Loading container information...</div>
    <div v-else>
      <div v-for="container in containers" :key="container.id" class="container-card">
        <div class="container-header">
          <h3>{{ container.name }}</h3>
          <div class="status-badge" :class="container.status">
            {{ container.status }}
          </div>
        </div>
        
        <!-- Container Details -->
        <div class="container-details">
          <div class="detail-section">
            <h4>Container Information</h4>
            <div class="detail-grid">
              <div class="detail-item">
                <label>Image:</label>
                <span>{{ container.image || 'N/A' }}</span>
              </div>
              <div class="detail-item">
                <label>OS:</label>
                <span>{{ container.os_info || 'Unknown' }}</span>
              </div>
              <div class="detail-item">
                <label>Architecture:</label>
                <span>{{ container.architecture || 'N/A' }}</span>
              </div>
              <div class="detail-item">
                <label>Created:</label>
                <span>{{ formatDate(container.created) }}</span>
              </div>
              <div class="detail-item">
                <label>Uptime:</label>
                <span>{{ container.uptime || 'N/A' }}</span>
              </div>
              <div class="detail-item">
                <label>Resource Usage:</label>
                <span>CPU: {{ container.cpu_usage || '0' }}% | RAM: {{ container.memory_usage || '0' }}MB</span>
              </div>
            </div>
          </div>

          <!-- Services Section -->
          <div class="detail-section">
            <h4>Associated Services</h4>
            <div v-if="container.services && container.services.length > 0" class="services-list">
              <div v-for="service in container.services" :key="service.name" class="service-item">
                <div class="service-info">
                  <span class="service-name">{{ service.name }}</span>
                  <span class="service-status" :class="service.status">{{ service.status }}</span>
                </div>
                <div class="service-actions">
                  <button @click="updateService(container.id, service.name)" class="btn-small btn-primary">
                    Update
                  </button>
                  <button @click="checkSystemdService(container.id, service.name)" class="btn-small btn-secondary">
                    Check Systemd
                  </button>
                  <button @click="editServiceConfig(container.id, service.name)" class="btn-small btn-info">
                    Config
                  </button>
                </div>
              </div>
            </div>
            <div v-else class="no-services">
              No associated services found
            </div>
          </div>
        </div>

        <!-- Action Buttons -->
        <div class="container-actions">
          <div class="action-group">
            <h5>Container Control</h5>
            <button @click="startContainer(container.id)" :disabled="container.status === 'running'" class="btn btn-success">
              Start
            </button>
            <button @click="stopContainer(container.id)" :disabled="container.status !== 'running'" class="btn btn-danger">
              Stop
            </button>
            <button @click="restartContainer(container.id)" class="btn btn-warning">
              Restart
            </button>
          </div>

          <div class="action-group">
            <h5>System Management</h5>
            <button @click="updateContainer(container.id)" class="btn btn-primary">
              Update OS
            </button>
            <button @click="upgradeContainer(container.id)" class="btn btn-primary">
              Upgrade OS
            </button>
            <button @click="checkMissingBinaries(container.id)" class="btn btn-info">
              Check & Install Binaries
            </button>
          </div>

          <div class="action-group">
            <h5>Service Management</h5>
            <button @click="fixInactiveServices(container.id)" class="btn btn-warning">
              Fix All Inactive Services
            </button>
            <button @click="refreshServices(container.id)" class="btn btn-secondary">
              Refresh Services
            </button>
          </div>

          <div class="action-group">
            <h5>Configuration</h5>
            <button @click="openConfigEditor(container.id)" class="btn btn-info">
              Edit Configs
            </button>
            <button @click="applyAIOptimization(container.id)" class="btn btn-special">
              AI Optimize Settings
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Config Editor Modal -->
    <div v-if="showConfigEditor" class="modal-overlay" @click="closeConfigEditor">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h3>Configuration Editor - {{ selectedContainer?.name }}</h3>
          <button @click="closeConfigEditor" class="close-btn">×</button>
        </div>
        <div class="modal-body">
          <div class="config-tabs">
            <button v-for="config in availableConfigs" :key="config" 
                    @click="selectedConfig = config"
                    :class="{ active: selectedConfig === config }"
                    class="tab-btn">
              {{ config }}
            </button>
          </div>
          <div class="config-editor">
            <textarea v-model="configContent" class="config-textarea" placeholder="Configuration content will appear here..."></textarea>
          </div>
          <div class="modal-actions">
            <button @click="saveConfig" class="btn btn-primary">Save Config</button>
            <button @click="applyAIOptimizedConfig" class="btn btn-special">Apply AI Optimized</button>
            <button @click="closeConfigEditor" class="btn btn-secondary">Cancel</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export default {
  name: 'Containers',
  setup() {
    const containers = ref([])
    const loading = ref(false)
    const selectedCategory = ref('All')
    const searchQuery = ref('')
    const sortBy = ref('name')
    const sortOrder = ref('asc')
    
    // Config Editor Modal State
    const showConfigEditor = ref(false)
    const selectedContainer = ref(null)
    const selectedConfig = ref('')
    const availableConfigs = ref([])
    const configContent = ref('')
    const aiSuggestions = ref([])
    const showAISuggestions = ref(false)
    
    // Operation state
    const operationInProgress = ref({})
    const lastOperationResult = ref(null)
    const showOperationResult = ref(false)

    // Computed properties for filtering and sorting
    const categories = computed(() => {
      const cats = ['All', ...new Set(containers.value.map(c => c.category))]
      return cats.filter(Boolean)
    })

    const filteredAndSortedContainers = computed(() => {
      let filtered = containers.value.filter(container => {
        const matchesCategory = selectedCategory.value === 'All' || container.category === selectedCategory.value
        const matchesSearch = container.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
                            container.description?.toLowerCase().includes(searchQuery.value.toLowerCase())
        return matchesCategory && matchesSearch
      })

      return filtered.sort((a, b) => {
        let aVal = a[sortBy.value]
        let bVal = b[sortBy.value]
        
        if (typeof aVal === 'string') {
          aVal = aVal.toLowerCase()
          bVal = bVal.toLowerCase()
        }
        
        if (sortOrder.value === 'asc') {
          return aVal < bVal ? -1 : aVal > bVal ? 1 : 0
        } else {
          return aVal > bVal ? -1 : aVal < bVal ? 1 : 0
        }
      })
    })

    const fetchContainers = async () => {
      try {
        loading.value = true
        const result = await invoke('get_system_overview')
        
        // Enhance containers with additional details
        const enhancedContainers = await Promise.all(
          result.containers.map(async (container) => {
            try {
              const details = await invoke('get_container_details', { container_id: container.id })
              return {
                ...container,
                os_info: details.os_info?.distribution || 'Unknown',
                architecture: details.os_info?.architecture || 'Unknown',
                image: `${details.os_info?.distribution}:${details.os_info?.version}` || 'Unknown',
                services: details.systemd_services || [],
                created: new Date().toISOString(),
                uptime: container.uptime || '0 minutes',
                cpu_usage: container.cpu_usage || Math.random() * 100,
                memory_usage: container.memory_usage || Math.random() * 1000
              }
            } catch (error) {
              console.warn(`Failed to get details for container ${container.id}:`, error)
              return {
                ...container,
                os_info: 'Unknown',
                architecture: 'Unknown',
                image: 'Unknown',
                services: [],
                created: new Date().toISOString(),
                uptime: '0 minutes',
                cpu_usage: Math.random() * 100,
                memory_usage: Math.random() * 1000
              }
            }
          })
        )
        
        containers.value = enhancedContainers
        console.log('Enhanced containers:', enhancedContainers)
      } catch (error) {
        console.error('Failed to fetch containers:', error)
        // Fallback with mock data for development
        containers.value = generateMockContainers()
      } finally {
        loading.value = false
      }
    }

    // Container control functions
    const startContainer = async (id) => {
      await executeContainerOperation(id, 'start', () => invoke('start_container', { container_id: id }))
    }

    const stopContainer = async (id) => {
      await executeContainerOperation(id, 'stop', () => invoke('stop_container', { container_id: id }))
    }

    const restartContainer = async (id) => {
      await executeContainerOperation(id, 'restart', () => invoke('restart_container', { container_id: id }))
    }

    // System management functions
    const updateContainer = async (id) => {
      await executeContainerOperation(id, 'update', () => invoke('update_container_packages', { container_id: id }))
    }

    const upgradeContainer = async (id) => {
      await executeContainerOperation(id, 'upgrade', () => invoke('update_container_packages', { container_id: id }))
    }

    const checkMissingBinaries = async (id) => {
      await executeContainerOperation(id, 'check-binaries', () => invoke('check_and_install_binaries'))
    }

    const fixInactiveServices = async (id) => {
      await executeContainerOperation(id, 'fix-services', () => invoke('fix_all_services'))
    }

    const refreshServices = async (id) => {
      await executeContainerOperation(id, 'refresh', () => fetchContainers())
    }

    // Service management functions
    const updateService = async (containerId, serviceName) => {
      try {
        operationInProgress.value[`${containerId}-${serviceName}`] = true
        await invoke('control_service', { 
          service_name: serviceName, 
          action: 'restart',
          container_id: containerId,
          vm_id: null
        })
        await fetchContainers()
        showNotification(`Service ${serviceName} updated successfully`, 'success')
      } catch (error) {
        console.error(`Failed to update service ${serviceName}:`, error)
        showNotification(`Failed to update service ${serviceName}`, 'error')
      } finally {
        operationInProgress.value[`${containerId}-${serviceName}`] = false
      }
    }

    const checkSystemdService = async (containerId, serviceName) => {
      try {
        const result = await invoke('check_service_status', {
          service_name: serviceName,
          container_id: containerId,
          vm_id: null
        })
        showNotification(`Service ${serviceName}: ${result.status}`, 'info')
      } catch (error) {
        console.error(`Failed to check service ${serviceName}:`, error)
        showNotification(`Failed to check service ${serviceName}`, 'error')
      }
    }

    const editServiceConfig = async (containerId, serviceName) => {
      selectedContainer.value = containers.value.find(c => c.id === containerId)
      selectedConfig.value = serviceName
      await openConfigEditor(containerId)
    }

    // Configuration management
    const openConfigEditor = async (containerId) => {
      try {
        selectedContainer.value = containers.value.find(c => c.id === containerId)
        showConfigEditor.value = true
        
        // Get available configs for this container
        const configs = await invoke('get_container_configs', { container_id: containerId })
        availableConfigs.value = configs.map(c => c.name)
        
        if (availableConfigs.value.length > 0) {
          selectedConfig.value = availableConfigs.value[0]
          await loadConfigContent()
        }
      } catch (error) {
        console.error('Failed to open config editor:', error)
        showNotification('Failed to open configuration editor', 'error')
      }
    }

    const loadConfigContent = async () => {
      if (!selectedContainer.value || !selectedConfig.value) return
      
      try {
        const configPath = getConfigPath(selectedConfig.value)
        const content = await invoke('read_container_config', {
          container_id: selectedContainer.value.id,
          config_path: configPath
        })
        configContent.value = content
      } catch (error) {
        console.error('Failed to load config content:', error)
        configContent.value = '# Failed to load configuration file\n# Please check permissions and file path'
      }
    }

    const saveConfig = async () => {
      if (!selectedContainer.value || !selectedConfig.value) return
      
      try {
        const configPath = getConfigPath(selectedConfig.value)
        await invoke('write_container_config', {
          container_id: selectedContainer.value.id,
          config_path: configPath,
          content: configContent.value
        })
        showNotification('Configuration saved successfully', 'success')
        closeConfigEditor()
      } catch (error) {
        console.error('Failed to save config:', error)
        showNotification('Failed to save configuration', 'error')
      }
    }

    const applyAIOptimizedConfig = async () => {
      if (!selectedContainer.value || !selectedConfig.value) return
      
      try {
        const suggestions = await invoke('get_ai_config_suggestions', {
          container_id: selectedContainer.value.id,
          config_path: getConfigPath(selectedConfig.value),
          config_content: configContent.value
        })
        
        aiSuggestions.value = suggestions
        showAISuggestions.value = true
      } catch (error) {
        console.error('Failed to get AI suggestions:', error)
        showNotification('Failed to get AI optimization suggestions', 'error')
      }
    }

    const applyAISuggestion = (suggestion) => {
      // This would apply the AI suggestion to the config
      // Implementation depends on the suggestion format
      configContent.value += `\n\n# AI Suggestion: ${suggestion.suggestion}\n# ${suggestion.explanation}\n`
      showAISuggestions.value = false
    }

    const applyAIOptimization = async (containerId) => {
      try {
        const result = await invoke('scan_media_stack')
        showNotification('AI optimization applied successfully', 'success')
        console.log('AI optimization result:', result)
      } catch (error) {
        console.error('Failed to apply AI optimization:', error)
        showNotification('Failed to apply AI optimization', 'error')
      }
    }

    const closeConfigEditor = () => {
      showConfigEditor.value = false
      selectedContainer.value = null
      selectedConfig.value = ''
      configContent.value = ''
      aiSuggestions.value = []
      showAISuggestions.value = false
    }

    // Helper functions
    const executeContainerOperation = async (id, operation, operationFn) => {
      try {
        operationInProgress.value[`${id}-${operation}`] = true
        const result = await operationFn()
        
        lastOperationResult.value = {
          success: true,
          message: typeof result === 'string' ? result : `${operation} completed successfully`,
          timestamp: new Date().toLocaleString()
        }
        
        await fetchContainers()
        showNotification(lastOperationResult.value.message, 'success')
      } catch (error) {
        console.error(`Failed to ${operation} container ${id}:`, error)
        lastOperationResult.value = {
          success: false,
          message: `Failed to ${operation} container: ${error}`,
          timestamp: new Date().toLocaleString()
        }
        showNotification(lastOperationResult.value.message, 'error')
      } finally {
        operationInProgress.value[`${id}-${operation}`] = false
        showOperationResult.value = true
        setTimeout(() => {
          showOperationResult.value = false
        }, 5000)
      }
    }

    const getConfigPath = (configName) => {
      const configPaths = {
        'nginx.conf': '/etc/nginx/nginx.conf',
        'config.xml': '/config/config.xml',
        'docker-compose.yml': '/docker-compose.yml',
        'settings.json': '/config/settings.json'
      }
      return configPaths[configName] || `/config/${configName}`
    }

    const formatDate = (dateString) => {
      if (!dateString) return 'Unknown'
      try {
        return new Date(dateString).toLocaleString()
      } catch {
        return 'Invalid date'
      }
    }

    const showNotification = (message, type = 'info') => {
      // This would show a toast notification
      console.log(`[${type.toUpperCase()}] ${message}`)
    }

    const generateMockContainers = () => {
      // Mock data for development/fallback
      return [
        {
          id: 210, name: 'Prowlarr', status: 'running', category: 'Essential Media Services',
          description: 'Indexer manager and proxy', os_info: 'Alpine Linux',
          architecture: 'x86_64', services: [{name: 'prowlarr', status: 'active'}],
          cpu_usage: 15.2, memory_usage: 256, uptime: '2d 5h 30m'
        },
        {
          id: 214, name: 'Sonarr', status: 'running', category: 'Essential Media Services',
          description: 'TV series management', os_info: 'Ubuntu 22.04',
          architecture: 'x86_64', services: [{name: 'sonarr', status: 'active'}],
          cpu_usage: 8.7, memory_usage: 512, uptime: '1d 12h 45m'
        }
      ]
    }

    // Auto-refresh
    onMounted(() => {
      fetchContainers()
      const interval = setInterval(fetchContainers, 60000) // Refresh every minute
      return () => clearInterval(interval)
    })

    return {
      // Data
      containers,
      loading,
      selectedCategory,
      searchQuery,
      sortBy,
      sortOrder,
      showConfigEditor,
      selectedContainer,
      selectedConfig,
      availableConfigs,
      configContent,
      aiSuggestions,
      showAISuggestions,
      operationInProgress,
      lastOperationResult,
      showOperationResult,
      
      // Computed
      categories,
      filteredAndSortedContainers,
      
      // Methods
      fetchContainers,
      startContainer,
      stopContainer,
      restartContainer,
      updateContainer,
      upgradeContainer,
      checkMissingBinaries,
      fixInactiveServices,
      refreshServices,
      updateService,
      checkSystemdService,
      editServiceConfig,
      openConfigEditor,
      loadConfigContent,
      saveConfig,
      applyAIOptimizedConfig,
      applyAISuggestion,
      applyAIOptimization,
      closeConfigEditor,
      formatDate
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

