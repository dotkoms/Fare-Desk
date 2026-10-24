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
    return { label: 'No data', tone: 'muted', title: 'No answer from the data plane yet' }
  }

  const title = [
    provenance.source,
    provenance.dataCardId,
    provenance.degraded ? provenance.degradedReason : null,
  ]
    .filter(Boolean)
    .join(' · ')

  switch (provenance.freshness) {
    case 'live':
      return { label: 'Live data', tone: 'ok', title }
    case 'cached':
      return { label: 'Cached data', tone: 'muted', title }
    case 'stale':
      return { label: 'Stale data', tone: 'warn', title }
    default:
      return { label: 'Reference data', tone: 'muted', title }
  }
}

export default function DataBadge(props: Props) {
  const { label, tone, title } = describe(props)
  return (
    <span className={`data-badge is-${tone}`} title={title}>
      {label}
    </span>
  )
}
