import {
  formatDistanceToNow,
  format,
  isToday,
  isYesterday,
  parseISO,
} from 'date-fns'

/**
 * Format a date string as a relative time (e.g. "5 minutes ago").
 */
export function timeAgo (dateString: string): string {
  const date = typeof dateString === 'string' ? parseISO(dateString) : dateString
  return formatDistanceToNow(date, { addSuffix: true })
}

/**
 * Format a date string for display in post/comment metadata.
 * Returns relative time for recent dates, absolute for older ones.
 */
export function formatDate (dateString: string): string {
  const date = parseISO(dateString)

  if (isToday(date)) {
    return timeAgo(dateString)
  }

  if (isYesterday(date)) {
    return `Yesterday at ${format(date, 'h:mm a')}`
  }

  return format(date, 'MMM d, yyyy')
}

/**
 * Format a date with full timestamp (used in tooltips, details).
 */
export function formatFullDate (dateString: string): string {
  return format(parseISO(dateString), 'MMMM d, yyyy \'at\' h:mm a')
}
