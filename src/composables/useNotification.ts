/**
 * 通知钩子函数
 */

import { ElMessage, ElMessageBox } from 'element-plus';
import type { MessageOptions } from 'element-plus';

/**
 * 创建通知钩子
 * @returns 通知相关函数
 */
export function useNotification() {
  /**
   * 显示成功消息
   * @param message 消息内容
   * @param options 消息选项
   */
  const showSuccess = (message: string, options?: MessageOptions) => {
    ElMessage.success({
      message,
      duration: 2000,
      ...options
    });
  };
  
  /**
   * 显示错误消息
   * @param message 消息内容
   * @param options 消息选项
   */
  const showError = (message: string, options?: MessageOptions) => {
    ElMessage.error({
      message,
      duration: 5000,
      ...options
    });
  };
  
  /**
   * 显示警告消息
   * @param message 消息内容
   * @param options 消息选项
   */
  const showWarning = (message: string, options?: MessageOptions) => {
    ElMessage.warning({
      message,
      duration: 3000,
      ...options
    });
  };
  
  /**
   * 显示确认对话框
   * @param title 标题
   * @param message 消息内容
   * @param options 对话框选项
   * @returns Promise，确认返回true，取消返回false
   */
  const showConfirm = (
    title: string, 
    message: string, 
    options?: {
      confirmButtonText?: string;
      cancelButtonText?: string;
      type?: 'success' | 'warning' | 'info' | 'error';
    }
  ): Promise<boolean> => {
    return ElMessageBox.confirm(message, title, {
      confirmButtonText: '确认',
      cancelButtonText: '取消',
      type: 'warning',
      ...options
    })
      .then(() => true)
      .catch(() => false);
  };
  
  /**
   * 显示输入对话框
   * @param title 标题
   * @param message 消息内容
   * @param options 对话框选项
   * @returns Promise，包含用户输入的值
   */
  const showPrompt = (
    title: string,
    message: string,
    options?: {
      inputPattern?: RegExp;
      inputErrorMessage?: string;
      inputValue?: string;
      confirmButtonText?: string;
      cancelButtonText?: string;
    }
  ): Promise<string | null> => {
    return ElMessageBox.prompt(message, title, {
      confirmButtonText: '确认',
      cancelButtonText: '取消',
      ...options
    })
      .then(({ value }) => value)
      .catch(() => null);
  };
  
  return {
    showSuccess,
    showError,
    showWarning,
    showConfirm,
    showPrompt
  };
} 