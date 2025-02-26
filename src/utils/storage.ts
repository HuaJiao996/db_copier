/**
 * 本地存储工具函数
 */

/**
 * 保存数据到本地存储
 * @param key 存储键
 * @param data 要存储的数据
 */
export function saveToStorage(key: string, data: any): void {
  try {
    const jsonData = JSON.stringify(data);
    localStorage.setItem(key, jsonData);
  } catch (error) {
    console.error('保存到本地存储失败:', error);
  }
}

/**
 * 从本地存储获取数据
 * @param key 存储键
 * @param defaultValue 默认值，如果没有找到数据则返回此值
 * @returns 存储的数据或默认值
 */
export function getFromStorage<T>(key: string, defaultValue: T): T {
  try {
    const data = localStorage.getItem(key);
    if (data) {
      return JSON.parse(data) as T;
    }
    return defaultValue;
  } catch (error) {
    console.error('从本地存储获取数据失败:', error);
    return defaultValue;
  }
}

/**
 * 从本地存储删除数据
 * @param key 存储键
 */
export function removeFromStorage(key: string): void {
  try {
    localStorage.removeItem(key);
  } catch (error) {
    console.error('从本地存储删除数据失败:', error);
  }
}

/**
 * 清空本地存储
 */
export function clearStorage(): void {
  try {
    localStorage.clear();
  } catch (error) {
    console.error('清空本地存储失败:', error);
  }
} 