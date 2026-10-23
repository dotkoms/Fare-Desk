import type { Provenance } from '../lib/api'

type Tone = 'muted' | 'ok' | 'warn' | 'danger'

type Props = {
  provenance: Provenance | null
  loading: boolean
  error: string | null
}

function describe({ provenance, loading, error }: Props): { label: string; tone: Tone; title: string } {
  if (error) {
    return { label: 'Data error', tone: 'danger', title: error }
  }
  if (loading && !provenance) {
    return { label: 'Connecting', tone: 'muted', title: 'Asking the data plane for prices' }
  }
  if (!provenance) {
    return { label: 'Demo data', tone: 'warn', title: 'Bundled demo fixtures' }
  }

  const title = [
    provenance.source,
    provenance.dataCardId,
    provenance.degraded ? provenance.degradedReason : null,
  ]
    .filter(Boolean)
    .join(' · ')

  // Fixtures stay demo data even when served from the data plane's cache.
  if (provenance.source === 'fixtures') {
    return { label: 'Demo data', tone: 'warn', title }
  }

  switch (provenance.freshness) {
    case 'live':
      return { label: 'Live data', tone: 'ok', title }
    case 'cached':
      return { label: 'Cached data', tone: 'muted', title }
    case 'stale':
      return { label: 'Stale data', tone: 'warn', title }
    default:
      return { label: 'Demo data', tone: 'warn', title }
  }
}

export default function DataBadge(props: Props) {
  const { label, tone, title } = describe(props)
  return (
    <span className={`demo-badge is-${tone}`} title={title}>
      {label}
    </span>
  )
}
