export default {
  nav: {
    taskManager: '任务管理',
    configManager: '配置管理'
  },
  common: {
    save: '保存',
    cancel: '取消',
    delete: '删除',
    copy: '复制',
    edit: '编辑',
    load: '加载',
    actions: '操作',
    refresh: '刷新',
    confirm: '确定',
    tips: '提示'
  },
  taskManager: {
    title: '任务管理',
    refresh: '刷新',
    exportRecords: '导出记录',
    taskList: '任务列表',
    runningCount: '{count} 个运行中',
    statusFilter: '状态筛选',
    searchTaskId: '搜索任务ID',
    taskDetails: '任务详情',
    copyProgress: '复制进度',
    currentTable: '当前表',
    details: '详情',
    status: {
      running: '运行中',
      completed: '已完成',
      failed: '失败',
      pending: '等待中',
      unknown: '未知'
    },
    columns: {
      taskId: '任务ID',
      status: '状态',
      progress: '进度',
      startTime: '开始时间',
      endTime: '结束时间',
      message: '消息',
      actions: '操作'
    }
  },
  configManager: {
    title: '配置管理',
    newConfig: '新建配置',
    createConfig: '创建配置',
    configList: '配置列表',
    importConfig: '导入配置',
    batchStart: '批量启动',
    startTask: '启动任务',
    copyConfig: '复制配置',
    importResult: '导入结果',
    noConfig: '暂无配置',
    selectedCount: '已选择 {count} 项',
    enterNewName: '请输入新配置名称',
    confirmDelete: '确定要删除该配置吗？',
    confirmBatchStart: '确定要启动选中的 {count} 个配置吗？',
    columns: {
      name: '配置名称',
      actions: '操作'
    },
    errors: {
      loadFailed: '加载配置失败',
      saveFailed: '保存配置失败',
      deleteFailed: '删除配置失败',
      copyFailed: '复制配置失败',
      taskCreateFailed: '创建任务失败',
      batchStartFailed: '批量启动任务失败',
      batchStartPartialFailed: '{count} 个任务启动失败',
      importFailed: '导入失败 {path}: {error}',
      noConfigSelected: '请选择要启动的配置',
      invalidName: '配置名称必须是2-50个字符（支持中文、字母、数字、下划线）'
    },
    messages: {
      saveSuccess: '保存配置成功',
      deleteSuccess: '删除配置成功',
      copySuccess: '复制配置成功',
      taskCreated: '任务创建成功',
      batchStartSuccess: '成功启动 {count} 个任务',
      importSuccess: '成功导入配置: {path}',
      importComplete: '导入完成。成功: {success}, 失败: {failed}\n{details}'
    }
  },
  tableConfig: {
    refreshTableList: '刷新表列表',
    structureOnly: '仅复制表结构',
    ignoreForeignKeys: '忽略外键关联',
    structureChanged: '表结构已变更',
    newColumns: '新增列',
    removedColumns: '移除列',
    updateStructure: '更新表结构',
    lastUpdated: '上次更新',
    refreshColumns: '刷新列',
    loadingColumnInfo: '加载列信息',
    default: '默认',
    new: '新',
    useHash: '使用哈希加密',
    selectRule: '选择规则',
    enterReplacement: '请输入替换值或模式',
    columnCount: '{count} 列',
    ruleCount: '{count} 规则',
    columns: {
      tableName: '表名',
      columnName: '列名',
      maskRule: '掩码规则'
    },
    rules: {
      none: '无',
      hash: '哈希',
      fixed: '固定值',
      pattern: '模式'
    }
  },
  databaseConfig: {
    source: '源数据库',
    target: '目标数据库',
    host: '主机地址',
    hostPlaceholder: '例如：localhost',
    port: '端口',
    database: '数据库名',
    username: '用户名',
    password: '密码',
    sslMode: 'SSL模式',
    sslModes: {
      prefer: '首选 (prefer)',
      require: '要求 (require)',
      disable: '禁用 (disable)'
    },
    testConnection: '测试连接',
    enableSSH: '启用 SSH 隧道',
    sshHost: 'SSH 主机',
    sshHostPlaceholder: 'SSH 服务器地址',
    sshPort: 'SSH 端口',
    sshUsername: 'SSH 用户名',
    authType: '认证方式',
    authTypes: {
      password: '密码',
      privateKey: '密钥'
    },
    privateKey: '私钥',
    selectPrivateKeyPlaceholder: '请选择私钥文件',
    selectFile: '选择文件',
    errors: {
      hostRequired: '主机地址不能为空',
      databaseRequired: '数据库名不能为空',
      usernameRequired: '用户名不能为空',
      connectionFailed: '连接失败: {error}',
      selectKeyFailed: '选择私钥文件失败'
    },
    messages: {
      connectionSuccess: '连接成功: {message}'
    }
  },
  configDetail: {
    title: {
      new: '新建配置',
      edit: '编辑配置: {name}'
    },
    configName: '配置名称',
    tabs: {
      connection: '数据库连接',
      tables: '表和列配置'
    },
    rules: {
      nameRequired: '请输入配置名称',
      nameLength: '长度在 2 到 50 个字符'
    },
    errors: {
      loadFailed: '加载配置失败: {error}',
      saveFailed: '保存配置失败: {error}',
      taskCreateFailed: '创建任务失败: {error}'
    },
    messages: {
      completeDbConfig: '请先完成数据库连接配置',
      saveSuccess: '保存配置成功',
      taskCreated: '任务创建成功',
      selectTables: '请选择要复制的表'
    }
  }
} 