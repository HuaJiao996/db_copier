/**
 * 加载状态钩子函数
 */

import { ref } from 'vue';

/**
 * 创建加载状态钩子
 * @returns 加载状态和控制函数
 */
export function useLoading() {
  const isLoading = ref(false);
  
  /**
   * 设置加载状态
   * @param value 加载状态值
   */
  const setLoading = (value: boolean) => {
    isLoading.value = value;
  };
  
  /**
   * 包装异步函数，自动处理加载状态
   * @param fn 要包装的异步函数
   * @returns 包装后的函数
   */
  const withLoading = async <T>(fn: () => Promise<T>): Promise<T> => {
    try {
      isLoading.value = true;
      return await fn();
    } finally {
      isLoading.value = false;
    }
  };
  
  /**
   * 执行异步操作并自动处理加载状态
   * @param fn 异步函数
   * @returns 异步函数的结果
   */
  const runWithLoading = async <T>(fn: () => Promise<T>): Promise<T> => {
    return withLoading(fn);
  };
  
  return {
    isLoading,
    setLoading,
    withLoading,
    runWithLoading
  };
} 