export default function Sparkline({ values }: { values: number[] }) {
  if (values.length < 2) return null

  const width = 320
  const height = 78
  const min = Math.min(...values)
  const max = Math.max(...values)
  const range = max - min || 1
  const points = values
    .map((value, index) => {
      const x = (index / (values.length - 1)) * width
      const y = height - ((value - min) / range) * (height - 10) - 5
      return `${x},${y}`
    })
    .join(' ')

  const last = values[values.length - 1]
  const lastY = height - ((last - min) / range) * (height - 10) - 5

  return (
    <svg className="sparkline" viewBox={`0 0 ${width} ${height}`} preserveAspectRatio="none" role="img" aria-label="Daily lowest fare on this route">
      <line x1="0" y1={height - 5} x2={width} y2={height - 5} className="sparkline-base" vectorEffect="non-scaling-stroke" />
      <polyline points={points} className="sparkline-line" vectorEffect="non-scaling-stroke" />
      <circle cx={width} cy={lastY} r="4" className="sparkline-dot" />
    </svg>
  )
}
