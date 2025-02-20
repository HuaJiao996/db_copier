import { ElMessage } from 'element-plus'

export function formatError(error: unknown): string {
  if (error instanceof Error) {
    return error.message
  }
  if (typeof error === 'string') {
    return error
  }
  return '未知错误'
}

export async function handleApiError<T>(
  promise: Promise<T>,
  errorMessage: string
): Promise<T> {
  try {
    return await promise
  } catch (error) {
    const message = formatError(error)
    ElMessage.error(`${errorMessage}: ${message}`)
    throw error
  }
} 