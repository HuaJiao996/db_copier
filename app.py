from flask import Flask, render_template, request, jsonify, flash, redirect, url_for
from werkzeug.utils import secure_filename
import yaml
import os
import json
from db_copier import DBCopier
import logging
import threading
from datetime import datetime
from sqlalchemy.orm import sessionmaker
from models import init_db, Config, TableConfig
import psycopg2
from sshtunnel import SSHTunnelForwarder

app = Flask(__name__)
app.secret_key = os.urandom(24)
app.config['UPLOAD_FOLDER'] = 'uploads'

# 确保上传目录存在
os.makedirs(app.config['UPLOAD_FOLDER'], exist_ok=True)

# 配置日志
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[
        logging.StreamHandler(),
        logging.FileHandler('web_app.log')
    ]
)
logger = logging.getLogger(__name__)

# 初始化数据库
engine = init_db()
Session = sessionmaker(bind=engine)

# 存储复制任务状态
copy_tasks = {}

def load_config():
    """加载默认配置"""
    try:
        session = Session()
        config = session.query(Config).first()
        if config:
            return config.to_dict()
    except Exception as e:
        logger.error(f"加载配置失败: {str(e)}")
    finally:
        session.close()
    
    return {
        'ssh': {
            'host': '',
            'port': 22,
            'username': '',
            'private_key_path': ''
        },
        'source_db': {
            'host': 'localhost',
            'port': 5432,
            'database': '',
            'username': '',
            'password': ''
        },
        'target_db': {
            'host': 'localhost',
            'port': 5432,
            'database': '',
            'username': '',
            'password': ''
        },
        'tables': []
    }

def run_copy_task(config_data, task_id):
    """运行数据库复制任务"""
    try:
        copy_tasks[task_id]['status'] = 'running'
        copy_tasks[task_id]['start_time'] = datetime.now().strftime('%Y-%m-%d %H:%M:%S')
        
        # 保存配置到临时文件
        temp_config_path = f'uploads/config_{task_id}.yaml'
        with open(temp_config_path, 'w', encoding='utf-8') as f:
            yaml.dump(config_data, f)
        
        # 执行复制
        with DBCopier(temp_config_path) as copier:
            copier.copy_all_tables()
        
        copy_tasks[task_id]['status'] = 'completed'
        copy_tasks[task_id]['end_time'] = datetime.now().strftime('%Y-%m-%d %H:%M:%S')
        copy_tasks[task_id]['message'] = '复制完成'
        
    except Exception as e:
        copy_tasks[task_id]['status'] = 'failed'
        copy_tasks[task_id]['end_time'] = datetime.now().strftime('%Y-%m-%d %H:%M:%S')
        copy_tasks[task_id]['message'] = f'错误: {str(e)}'
        logger.error(f'Task {task_id} failed: {str(e)}')
    
    finally:
        # 清理临时配置文件
        if os.path.exists(temp_config_path):
            os.remove(temp_config_path)

@app.route('/')
def index():
    """主页"""
    config = load_config()
    return render_template('index.html', config=config)

@app.route('/save_config', methods=['POST'])
def save_config():
    """保存配置"""
    try:
        config_data = request.get_json()
        session = Session()
        
        # 查找或创建配置
        if 'name' in config_data:
            config = Config(name=config_data['name'])
        else:
            config = session.query(Config).first()
            if not config:
                config = Config(name='default')
        
        # 更新配置
        new_config = Config.from_dict(config_data)
        new_config.name = config.name
        
        session.merge(new_config)
        session.commit()
        
        return jsonify({'status': 'success', 'message': '配置已保存'})
    except Exception as e:
        logger.error(f"保存配置失败: {str(e)}")
        return jsonify({'status': 'error', 'message': str(e)})
    finally:
        session.close()

@app.route('/test_ssh', methods=['POST'])
def test_ssh():
    """测试SSH连接"""
    try:
        ssh_config = request.get_json()
        
        # 创建SSH隧道
        with SSHTunnelForwarder(
            (ssh_config['host'], int(ssh_config['port'])),
            ssh_username=ssh_config['username'],
            ssh_private_key=ssh_config.get('private_key_path'),
            ssh_password=ssh_config.get('password'),
            remote_bind_address=('localhost', 22)
        ) as tunnel:
            return jsonify({'status': 'success', 'message': 'SSH连接成功'})
    except Exception as e:
        return jsonify({'status': 'error', 'message': str(e)})

@app.route('/test_db_connection', methods=['POST'])
def test_db_connection():
    """测试数据库连接"""
    try:
        data = request.get_json()
        db_type = data['type']  # source 或 target
        db_config = data['config']
        
        # 连接数据库
        conn = psycopg2.connect(
            host=db_config['host'],
            port=int(db_config['port']),
            database=db_config['database'],
            user=db_config['username'],
            password=db_config['password']
        )
        
        # 测试连接
        with conn.cursor() as cursor:
            cursor.execute('SELECT 1')
        
        conn.close()
        return jsonify({'status': 'success', 'message': '数据库连接成功'})
    except Exception as e:
        return jsonify({'status': 'error', 'message': str(e)})

@app.route('/start_copy', methods=['POST'])
def start_copy():
    """开始复制任务"""
    try:
        config_data = request.get_json()
        
        # 创建任务ID
        task_id = datetime.now().strftime('%Y%m%d%H%M%S')
        
        # 初始化任务状态
        copy_tasks[task_id] = {
            'status': 'pending',
            'start_time': None,
            'end_time': None,
            'message': '任务初始化中'
        }
        
        # 启动复制线程
        thread = threading.Thread(target=run_copy_task, args=(config_data, task_id))
        thread.start()
        
        return jsonify({
            'status': 'success',
            'message': '复制任务已启动',
            'task_id': task_id
        })
    except Exception as e:
        return jsonify({'status': 'error', 'message': str(e)})

@app.route('/task_status/<task_id>')
def task_status(task_id):
    """获取任务状态"""
    if task_id in copy_tasks:
        return jsonify(copy_tasks[task_id])
    return jsonify({'status': 'not_found', 'message': '任务不存在'})

@app.route('/fetch_tables', methods=['POST'])
def fetch_tables():
    """从数据库获取表信息"""
    try:
        config_data = request.get_json()
        
        # 保存临时配置
        temp_config_path = f'uploads/temp_config_{datetime.now().strftime("%Y%m%d%H%M%S")}.yaml'
        with open(temp_config_path, 'w', encoding='utf-8') as f:
            yaml.dump(config_data, f)
        
        # 获取表信息
        with DBCopier(temp_config_path) as copier:
            tables_info = copier.get_tables_info(temp_config_path)
        
        # 清理临时文件
        os.remove(temp_config_path)
        
        return jsonify({
            'status': 'success',
            'tables': tables_info
        })
    except Exception as e:
        return jsonify({'status': 'error', 'message': str(e)})

@app.route('/list_configs')
def list_configs():
    """列出所有配置"""
    try:
        session = Session()
        configs = session.query(Config).all()
        return jsonify({
            'status': 'success',
            'configs': [{'id': c.id, 'name': c.name} for c in configs]
        })
    except Exception as e:
        return jsonify({'status': 'error', 'message': str(e)})
    finally:
        session.close()

@app.route('/load_config/<int:config_id>')
def load_config_by_id(config_id):
    """加载指定配置"""
    try:
        session = Session()
        config = session.query(Config).get(config_id)
        if config:
            return jsonify({
                'status': 'success',
                'config': config.to_dict()
            })
        return jsonify({'status': 'error', 'message': '配置不存在'})
    except Exception as e:
        return jsonify({'status': 'error', 'message': str(e)})
    finally:
        session.close()

if __name__ == '__main__':
    app.run(debug=True) 