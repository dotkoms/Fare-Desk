import { useEffect, useId, useRef, useState } from 'react'
import { placeByCode, searchPlaces, type Place } from '../lib/api'

type Props = {
  label: string
  code: string
  onChange: (code: string, place: Place) => void
}

const describe = (place: Place) => `${place.code} · ${place.city}`

/** An airport combobox backed by /api/places: type a code, city or airport name. */
export default function PlaceField({ label, code, onChange }: Props) {
  const id = useId()
  const [text, setText] = useState(code)
  const [selected, setSelected] = useState<Place | null>(null)
  const [options, setOptions] = useState<Place[]>([])
  const [open, setOpen] = useState(false)
  const [active, setActive] = useState(0)
  const editing = useRef(false)

  // Show the city for the current code, including codes that arrive from outside (swap, restore).
  useEffect(() => {
    if (selected?.code === code) return
    let cancelled = false
    placeByCode(code)
      .then((place) => {
        if (cancelled || editing.current) return
        setSelected(place)
        setText(place ? describe(place) : code)
      })
      .catch(() => {
        if (!cancelled && !editing.current) setText(code)
      })
    return () => {
      cancelled = true
    }
  }, [code, selected])

  useEffect(() => {
    const query = text.trim()
    if (!open || query.length < 2) return
    const controller = new AbortController()
    const timer = window.setTimeout(() => {
      searchPlaces(query, controller.signal)
        .then((places) => {
          setOptions(places)
          setActive(0)
        })
        .catch(() => {
          if (!controller.signal.aborted) setOptions([])
        })
    }, 120)
    return () => {
      window.clearTimeout(timer)
      controller.abort()
    }
  }, [text, open])

  function choose(place: Place) {
    editing.current = false
    setSelected(place)
    setText(describe(place))
    setOpen(false)
    onChange(place.code, place)
  }

  function restore() {
    editing.current = false
    setOpen(false)
    setText(selected ? describe(selected) : code)
  }

  const listId = `${id}-list`
  const showList = open && text.trim().length >= 2 && options.length > 0

  return (
    <div className="field place-field">
      <label className="field-label" htmlFor={id}>
        {label}
      </label>
      <input
        id={id}
        role="combobox"
        aria-expanded={showList}
        aria-controls={listId}
        aria-autocomplete="list"
        aria-activedescendant={showList ? `${id}-option-${active}` : undefined}
        autoComplete="off"
        spellCheck={false}
        value={text}
        onFocus={(event) => {
          editing.current = true
          event.currentTarget.select()
        }}
        onChange={(event) => {
          editing.current = true
          setText(event.target.value)
          setOpen(true)
        }}
        onBlur={() => window.setTimeout(restore, 120)}
        onKeyDown={(event) => {
          if (event.key === 'ArrowDown' && showList) {
            event.preventDefault()
            setActive((index) => Math.min(index + 1, options.length - 1))
          } else if (event.key === 'ArrowUp' && showList) {
            event.preventDefault()
            setActive((index) => Math.max(index - 1, 0))
          } else if (event.key === 'Enter' && showList) {
            event.preventDefault()
            choose(options[active])
          } else if (event.key === 'Escape') {
            restore()
          }
        }}
      />
      {showList ? (
        <ul className="place-options" id={listId} role="listbox" aria-label={`${label} airports`}>
          {options.map((place, index) => (
            <li
              key={place.id}
              id={`${id}-option-${index}`}
              role="option"
              aria-selected={index === active}
              className={index === active ? 'is-active' : ''}
              onMouseDown={(event) => {
                event.preventDefault()
                choose(place)
              }}
              onMouseEnter={() => setActive(index)}
            >
              <strong>{place.code}</strong>
              <span>
                {place.city}, {place.country}
              </span>
              <small>{place.name}</small>
            </li>
          ))}
        </ul>
      ) : null}
    </div>
  )
}
