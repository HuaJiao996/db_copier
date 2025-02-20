from sqlalchemy import create_engine, Column, Integer, String, Text, DateTime, ForeignKey
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import relationship
from datetime import datetime
import json

Base = declarative_base()

class Config(Base):
    """配置表"""
    __tablename__ = 'configs'
    
    id = Column(Integer, primary_key=True)
    name = Column(String(100), nullable=False, unique=True)  # 配置名称
    ssh_host = Column(String(255))
    ssh_port = Column(Integer)
    ssh_username = Column(String(100))
    ssh_private_key_path = Column(String(255))
    
    source_db_host = Column(String(255))
    source_db_port = Column(Integer)
    source_db_name = Column(String(100))
    source_db_username = Column(String(100))
    source_db_password = Column(String(100))
    
    target_db_host = Column(String(255))
    target_db_port = Column(Integer)
    target_db_name = Column(String(100))
    target_db_username = Column(String(100))
    target_db_password = Column(String(100))
    
    created_at = Column(DateTime, default=datetime.now)
    updated_at = Column(DateTime, default=datetime.now, onupdate=datetime.now)
    
    tables = relationship("TableConfig", back_populates="config", cascade="all, delete-orphan")
    
    def to_dict(self):
        """转换为字典格式"""
        return {
            'ssh': {
                'host': self.ssh_host,
                'port': self.ssh_port,
                'username': self.ssh_username,
                'private_key_path': self.ssh_private_key_path
            },
            'source_db': {
                'host': self.source_db_host,
                'port': self.source_db_port,
                'database': self.source_db_name,
                'username': self.source_db_username,
                'password': self.source_db_password
            },
            'target_db': {
                'host': self.target_db_host,
                'port': self.target_db_port,
                'database': self.target_db_name,
                'username': self.target_db_username,
                'password': self.target_db_password
            },
            'tables': [table.to_dict() for table in self.tables]
        }
    
    @classmethod
    def from_dict(cls, data):
        """从字典创建实例"""
        config = cls(
            ssh_host=data['ssh']['host'],
            ssh_port=data['ssh']['port'],
            ssh_username=data['ssh']['username'],
            ssh_private_key_path=data['ssh']['private_key_path'],
            
            source_db_host=data['source_db']['host'],
            source_db_port=data['source_db']['port'],
            source_db_name=data['source_db']['database'],
            source_db_username=data['source_db']['username'],
            source_db_password=data['source_db']['password'],
            
            target_db_host=data['target_db']['host'],
            target_db_port=data['target_db']['port'],
            target_db_name=data['target_db']['database'],
            target_db_username=data['target_db']['username'],
            target_db_password=data['target_db']['password']
        )
        
        for table_data in data.get('tables', []):
            config.tables.append(TableConfig.from_dict(table_data))
        
        return config

class TableConfig(Base):
    """表配置"""
    __tablename__ = 'table_configs'
    
    id = Column(Integer, primary_key=True)
    config_id = Column(Integer, ForeignKey('configs.id'))
    table_name = Column(String(100), nullable=False)
    columns = Column(Text)  # JSON格式存储列名列表
    mask_rules = Column(Text)  # JSON格式存储脱敏规则
    
    config = relationship("Config", back_populates="tables")
    
    def to_dict(self):
        """转换为字典格式"""
        return {
            'name': self.table_name,
            'columns': json.loads(self.columns) if self.columns else [],
            'mask_rules': json.loads(self.mask_rules) if self.mask_rules else []
        }
    
    @classmethod
    def from_dict(cls, data):
        """从字典创建实例"""
        return cls(
            table_name=data['name'],
            columns=json.dumps(data.get('columns', [])),
            mask_rules=json.dumps(data.get('mask_rules', []))
        )

# 创建数据库引擎和表
def init_db():
    """初始化数据库"""
    engine = create_engine('sqlite:///dbcopy.db')
    Base.metadata.create_all(engine)
    return engine 