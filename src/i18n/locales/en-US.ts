export default {
  nav: {
    taskManager: 'Task Manager',
    configManager: 'Config Manager'
  },
  common: {
    save: 'Save',
    cancel: 'Cancel',
    delete: 'Delete',
    copy: 'Copy',
    edit: 'Edit',
    load: 'Load',
    actions: 'Actions',
    refresh: 'Refresh',
    confirm: 'Confirm',
    tips: 'Tips'
  },
  taskManager: {
    title: 'Task Manager',
    refresh: 'Refresh',
    exportRecords: 'Export Records',
    taskList: 'Task List',
    runningCount: '{count} Running',
    statusFilter: 'Status Filter',
    searchTaskId: 'Search Task ID',
    taskDetails: 'Task Details',
    copyProgress: 'Copy Progress',
    currentTable: 'Current Table',
    details: 'Details',
    status: {
      running: 'Running',
      completed: 'Completed',
      failed: 'Failed',
      pending: 'Pending',
      unknown: 'Unknown'
    },
    columns: {
      taskId: 'Task ID',
      status: 'Status',
      progress: 'Progress',
      startTime: 'Start Time',
      endTime: 'End Time',
      message: 'Message',
      actions: 'Actions'
    }
  },
  configManager: {
    title: 'Configuration Manager',
    newConfig: 'New Configuration',
    createConfig: 'Create Configuration',
    configList: 'Configuration List',
    importConfig: 'Import Configuration',
    batchStart: 'Batch Start',
    startTask: 'Start Task',
    copyConfig: 'Copy Configuration',
    importResult: 'Import Result',
    noConfig: 'No configurations',
    selectedCount: '{count} Selected',
    enterNewName: 'Please enter new configuration name',
    confirmDelete: 'Are you sure you want to delete this configuration?',
    confirmBatchStart: 'Are you sure you want to start {count} selected configurations?',
    columns: {
      name: 'Configuration Name',
      actions: 'Actions'
    },
    errors: {
      loadFailed: 'Failed to load configurations',
      saveFailed: 'Failed to save configuration',
      deleteFailed: 'Failed to delete configuration',
      copyFailed: 'Failed to copy configuration',
      taskCreateFailed: 'Failed to create task',
      batchStartFailed: 'Failed to start batch tasks',
      batchStartPartialFailed: '{count} tasks failed to start',
      importFailed: 'Import failed {path}: {error}',
      noConfigSelected: 'Please select configurations to start',
      invalidName: 'Configuration name must be 2-50 characters (supports letters, numbers, underscore and Chinese characters)'
    },
    messages: {
      saveSuccess: 'Configuration saved successfully',
      deleteSuccess: 'Configuration deleted successfully',
      copySuccess: 'Configuration copied successfully',
      taskCreated: 'Task created successfully',
      batchStartSuccess: 'Successfully started {count} tasks',
      importSuccess: 'Successfully imported configuration: {path}',
      importComplete: 'Import completed. Success: {success}, Failed: {failed}\n{details}'
    }
  },
  tableConfig: {
    refreshTableList: 'Refresh Table List',
    structureOnly: 'Structure Only',
    ignoreForeignKeys: 'Ignore Foreign Keys',
    structureChanged: 'Structure Changed',
    newColumns: 'New Columns',
    removedColumns: 'Removed Columns',
    updateStructure: 'Update Structure',
    lastUpdated: 'Last Updated',
    refreshColumns: 'Refresh Columns',
    loadingColumnInfo: 'Loading column information',
    default: 'Default',
    new: 'New',
    useHash: 'Use Hash Encryption',
    selectRule: 'Select Rule',
    enterReplacement: 'Enter replacement value or pattern',
    columnCount: '{count} Columns',
    ruleCount: '{count} Rules',
    columns: {
      tableName: 'Table Name',
      columnName: 'Column Name',
      maskRule: 'Mask Rule'
    },
    rules: {
      none: 'None',
      hash: 'Hash',
      fixed: 'Fixed Value',
      pattern: 'Pattern'
    }
  },
  databaseConfig: {
    source: 'Source Database',
    target: 'Target Database',
    host: 'Host',
    hostPlaceholder: 'e.g. localhost',
    port: 'Port',
    database: 'Database',
    username: 'Username',
    password: 'Password',
    sslMode: 'SSL Mode',
    sslModes: {
      prefer: 'Prefer',
      require: 'Require',
      disable: 'Disable'
    },
    testConnection: 'Test Connection',
    enableSSH: 'Enable SSH Tunnel',
    sshHost: 'SSH Host',
    sshHostPlaceholder: 'SSH Server Address',
    sshPort: 'SSH Port',
    sshUsername: 'SSH Username',
    authType: 'Authentication Type',
    authTypes: {
      password: 'Password',
      privateKey: 'Private Key'
    },
    privateKey: 'Private Key',
    selectPrivateKeyPlaceholder: 'Please select private key file',
    selectFile: 'Select File',
    errors: {
      hostRequired: 'Host is required',
      databaseRequired: 'Database name is required',
      usernameRequired: 'Username is required',
      connectionFailed: 'Connection failed: {error}',
      selectKeyFailed: 'Failed to select private key file'
    },
    messages: {
      connectionSuccess: 'Connection successful: {message}'
    }
  },
  configDetail: {
    title: {
      new: 'New Configuration',
      edit: 'Edit Configuration: {name}'
    },
    configName: 'Configuration Name',
    tabs: {
      connection: 'Database Connection',
      tables: 'Tables and Columns'
    },
    rules: {
      nameRequired: 'Please enter configuration name',
      nameLength: 'Length should be 2 to 50 characters'
    },
    errors: {
      loadFailed: 'Failed to load configuration: {error}',
      saveFailed: 'Failed to save configuration: {error}',
      taskCreateFailed: 'Failed to create task: {error}'
    },
    messages: {
      completeDbConfig: 'Please complete database connection configuration first',
      saveSuccess: 'Configuration saved successfully',
      taskCreated: 'Task created successfully',
      selectTables: 'Please select tables to copy'
    }
  }
} 