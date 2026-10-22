export default function GlobeFallback({ reason }: { reason?: string }) {
  return (
    <div className="globe-fallback">
      <svg viewBox="0 0 200 200" className="fallback-globe" aria-hidden="true">
        <circle cx="100" cy="100" r="78" fill="#0A1724" stroke="#2A4054" strokeWidth="1" />
        <ellipse cx="100" cy="100" rx="78" ry="30" fill="none" stroke="#223547" strokeWidth="1" />
        <ellipse cx="100" cy="100" rx="30" ry="78" fill="none" stroke="#223547" strokeWidth="1" />
        <path d="M40 122 Q100 44 160 92" fill="none" stroke="#6F879D" strokeWidth="1.5" />
        <circle cx="40" cy="122" r="3" fill="#E8EEF5" />
        <circle cx="160" cy="92" r="3" fill="#E8EEF5" />
      </svg>
      <div>
        <span className="stage-kicker">3D view unavailable</span>
        <h3>WebGL could not start.</h3>
        <p>
          {reason ?? 'The globe needs WebGL. The list, route strip and fare desk still work.'}
        </p>
      </div>
    </div>
  )
}
