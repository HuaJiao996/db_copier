import yaml
import psycopg2
from sshtunnel import SSHTunnelForwarder
import pandas as pd
from typing import Dict, List, Optional
import hashlib
import random
import string
from tqdm import tqdm
import logging
import sys

# 配置日志
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[
        logging.StreamHandler(sys.stdout),
        logging.FileHandler('db_copy.log')
    ]
)
logger = logging.getLogger(__name__)

class DataMasker:
    @staticmethod
    def partial_mask(value: str) -> str:
        """部分掩码，保留首尾字符"""
        if not value or len(value) <= 2:
            return value
        return value[0] + '*' * (len(value) - 2) + value[-1]

    @staticmethod
    def hash_mask(value: str) -> str:
        """哈希掩码"""
        return hashlib.md5(str(value).encode()).hexdigest()

    @staticmethod
    def random_mask(value: str) -> str:
        """随机字符替换"""
        length = len(str(value))
        return ''.join(random.choice(string.ascii_letters + string.digits) for _ in range(length))

class DBCopier:
    def __init__(self, config_path: str):
        with open(config_path, 'r', encoding='utf-8') as f:
            self.config = yaml.safe_load(f)
        
        self.ssh_tunnel = None
        self.source_conn = None
        self.target_conn = None
        self.masker = DataMasker()

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.cleanup()

    def setup_ssh_tunnel(self) -> None:
        """建立SSH隧道"""
        ssh_config = self.config['ssh']
        db_config = self.config['source_db']
        
        logger.info("正在建立SSH隧道...")
        self.ssh_tunnel = SSHTunnelForwarder(
            (ssh_config['host'], ssh_config.get('port', 22)),
            ssh_username=ssh_config['username'],
            ssh_private_key=ssh_config.get('private_key_path'),
            ssh_password=ssh_config.get('password'),
            remote_bind_address=(db_config['host'], db_config['port'])
        )
        self.ssh_tunnel.start()
        logger.info(f"SSH隧道已建立，本地端口: {self.ssh_tunnel.local_bind_port}")

    def connect_databases(self) -> None:
        """连接源数据库和目标数据库"""
        source_config = self.config['source_db']
        target_config = self.config['target_db']

        # 连接源数据库（通过SSH隧道）
        logger.info("正在连接源数据库...")
        self.source_conn = psycopg2.connect(
            host='localhost',
            port=self.ssh_tunnel.local_bind_port,
            database=source_config['database'],
            user=source_config['username'],
            password=source_config['password']
        )

        # 连接目标数据库
        logger.info("正在连接目标数据库...")
        self.target_conn = psycopg2.connect(
            host=target_config['host'],
            port=target_config['port'],
            database=target_config['database'],
            user=target_config['username'],
            password=target_config['password']
        )

    def apply_mask_rules(self, df: pd.DataFrame, mask_rules: List[Dict]) -> pd.DataFrame:
        """应用脱敏规则"""
        for rule in mask_rules:
            column = rule['column']
            method = rule['method']
            
            if column not in df.columns:
                continue

            mask_func = {
                'partial': self.masker.partial_mask,
                'hash': self.masker.hash_mask,
                'random': self.masker.random_mask
            }.get(method)

            if mask_func:
                df[column] = df[column].astype(str).apply(mask_func)

        return df

    def copy_table(self, table_config: Dict) -> None:
        """复制单个表"""
        table_name = table_config['name']
        columns = table_config.get('columns', [])
        mask_rules = table_config.get('mask_rules', [])

        logger.info(f"正在复制表 {table_name}...")

        # 构建查询语句
        columns_str = ', '.join(columns) if columns else '*'
        query = f"SELECT {columns_str} FROM {table_name}"

        # 读取源数据
        df = pd.read_sql(query, self.source_conn)
        
        # 应用脱敏规则
        if mask_rules:
            df = self.apply_mask_rules(df, mask_rules)

        # 写入目标数据库
        with self.target_conn.cursor() as cursor:
            # 获取表结构
            cursor.execute(f"""
                SELECT column_name, data_type 
                FROM information_schema.columns 
                WHERE table_name = '{table_name}'
            """)
            columns_info = cursor.fetchall()
            
            # 创建目标表
            columns_def = [f"{col[0]} {col[1]}" for col in columns_info]
            create_table_sql = f"""
                DROP TABLE IF EXISTS {table_name};
                CREATE TABLE {table_name} (
                    {', '.join(columns_def)}
                )
            """
            cursor.execute(create_table_sql)

        # 使用 to_sql 写入数据
        df.to_sql(
            table_name,
            self.target_conn,
            if_exists='append',
            index=False,
            method='multi',
            chunksize=1000
        )

        self.target_conn.commit()
        logger.info(f"表 {table_name} 复制完成，共 {len(df)} 行")

    def copy_all_tables(self) -> None:
        """复制所有配置的表"""
        try:
            self.setup_ssh_tunnel()
            self.connect_databases()

            for table_config in tqdm(self.config['tables']):
                self.copy_table(table_config)

        except Exception as e:
            logger.error(f"发生错误: {str(e)}")
            raise
        finally:
            self.cleanup()

    def cleanup(self) -> None:
        """清理资源"""
        if self.source_conn:
            self.source_conn.close()
        if self.target_conn:
            self.target_conn.close()
        if self.ssh_tunnel:
            self.ssh_tunnel.close()

if __name__ == '__main__':
    with DBCopier('config.yaml') as copier:
        copier.copy_all_tables() 