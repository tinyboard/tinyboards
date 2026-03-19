/**
 * Convert an RGB string (e.g. "34,197,94" or "34 197 94") to hex color.
 */
export function toHexCode (rgb: string): string {
  const values = rgb.replace(/\s+/g, ',').split(',').map(Number)
  return '#' + values.map(v => v.toString(16).padStart(2, '0')).join('')
}

/**
 * Convert a hex color (e.g. "#22c55e") to an RGB string (e.g. "34,197,94").
 */
export function toRGB (hex: string): string {
  const h = hex.slice(1)
  const values = [h.slice(0, 2), h.slice(2, 4), h.slice(4, 6)]
  return values.map(v => parseInt(v, 16)).join(',')
}

/**
 * Convert an RGB string from the backend into valid CSS.
 * Handles hex, comma-separated RGB, and space-separated RGB.
 */
export function toCSSColor (rgbString: string): string {
  if (!rgbString) { return '' }

  if (rgbString.startsWith('#')) {
    return rgbString
  }

  if (rgbString.includes(',')) {
    return `rgb(${rgbString})`
  }

  // Space-separated values (CSS custom property format)
  if (/^\d+\s+\d+\s+\d+$/.test(rgbString.trim())) {
    return `rgb(${rgbString.trim().replace(/\s+/g, ', ')})`
  }

  return rgbString
}
