/**
 * 错误处理工具
 */

import { useNotification } from '@/composables/useNotification';

/**
 * 格式化错误消息
 * @param error 错误对象
 * @returns 格式化后的错误消息
 */
export function formatError(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  } else if (typeof error === 'string') {
    return error;
  } else if (error === undefined || error === null) {
    return '未知错误';
  } else {
    try {
      return String(error);
    } catch {
      return '未知错误';
    }
  }
}

/**
 * 处理API错误
 * @param error 错误对象
 * @param errorMessage 错误前缀
 */
export function handleApiError(error: unknown, errorMessage: string = '操作失败'): void {
  const { showError } = useNotification();
  const message = formatError(error);
  showError(`${errorMessage}: ${message}`);
}

/**
 * 包装异步函数，自动处理错误
 * @param fn 异步函数
 * @param errorMessage 错误前缀
 * @returns 包装后的函数
 */
export function withErrorHandling<T>(
  fn: () => Promise<T>,
  errorMessage: string = '操作失败'
): Promise<T | null> {
  return fn().catch((error) => {
    handleApiError(error, errorMessage);
    return null;
  });
} 