import { Component, type ReactNode } from 'react'
import GlobeFallback from './GlobeFallback'

type Props = { children: ReactNode }
type State = { failed: boolean }

export default class GlobeBoundary extends Component<Props, State> {
  state: State = { failed: false }

  static getDerivedStateFromError(): State {
    return { failed: true }
  }

  render() {
    if (this.state.failed) {
      return <GlobeFallback reason="The 3D renderer failed to start. The list and route strip are still usable." />
    }
    return this.props.children
  }
}
