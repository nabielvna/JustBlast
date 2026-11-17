<script lang="ts">
  import type {
    DeviceView,
    DeviceAction,
    Preferences,
    WeightsView,
    AutomationSettings,
    HistoryEntry,
  } from '../../generated/contracts'
  import CalibrationPanel from './CalibrationPanel.svelte'
  import Icon from '../../components/ui/Icon.svelte'
  import { preferencesApi } from '../../lib/native/preferences'
  let {
    disabled,
    device,
    preferences,
    onpreferences,
    ondevice,
    weights,
    onweights,
  }: {
    disabled: boolean
    device: DeviceView | undefined
    preferences: Preferences | null
    onpreferences: (prefs: Preferences) => Promise<Preferences>
    ondevice: (action: DeviceAction) => Promise<void>
    weights: WeightsView | undefined
    onweights: (weights: WeightsView) => Promise<void>
  } = $props()
  let dialog: HTMLDialogElement
  type Section = 'connection' | 'automation' | 'scoring' | 'history' | 'calibration'
  const sections: { key: Section; label: string }[] = [
    { key: 'connection', label: 'Phone connection' },
    { key: 'calibration', label: 'Phone calibration' },
    { key: 'automation', label: 'Automation' },
    { key: 'scoring', label: 'Algorithm scoring' },
    { key: 'history', label: 'Cycle history' },
  ]
  let tab = $state<Section>('connection')
  let pairingAddress = $state('')
  let pairingOpen = $state(false)
  let draft = $state<Preferences | null>(null),
    saving = $state(false),
    message = $state(''),
    code = $state(''),
    install = $state(false)
  let calibrating = $state(false)
  let history = $state<HistoryEntry[]>([])
  const id = $props.id()
  let current = $derived(
    draft?.connection.serial ||
      device?.serial ||
      device?.devices.find((d) => d.state === 'device')?.serial ||
      '',
  )
  let busy = $derived(disabled || saving || calibrating || !!device?.busy)
  export function show(section?: 'connection' | 'automation') {
    if (section) tab = section
    draft = preferences ? structuredClone($state.snapshot(preferences)) : null
    message = ''
    code = ''
    pairingAddress = ''
    pairingOpen = false
    saving = true
    dialog.showModal()
    void preferencesApi
      .snapshot()
      .then((v) => {
        if (dialog.open) draft = v
      })
      .catch((e: unknown) => {
        message = e instanceof Error ? e.message : 'Could not load preferences.'
      })
      .finally(() => {
        saving = false
      })
  }
  async function save() {
    if (!draft || saving) return
    saving = true
    message = ''
    try {
      draft = await onpreferences($state.snapshot(draft))
      message = 'Preferences saved.'
    } catch (e) {
      message = e instanceof Error ? e.message : 'Could not save preferences.'
    } finally {
      saving = false
    }
  }
  async function wireless(pair: boolean) {
    if (!draft || busy) return
    message = ''
    saving = true
    try {
      await ondevice({
        type: 'wireless',
        address: (pair ? pairingAddress : draft.connection.wirelessAddress).trim(),
        code: pair ? code.trim() : null,
      })
      if (pair) {
        code = ''
      } else {
        draft = await onpreferences($state.snapshot(draft))
      }
    } catch (reason) {
      message = reason instanceof Error ? reason.message : 'Could not connect wirelessly.'
    } finally {
      saving = false
    }
  }
  function selectTab(section: Section) {
    tab = section
    dialog.querySelector('.settings-body')?.scrollTo({ top: 0 })
    if (section === 'history') void loadHistory()
  }
  async function importCalibration(event: Event) {
    const file = (event.currentTarget as HTMLInputElement).files?.[0]
    if (!file || file.size > 65536) {
      message = 'Choose a TOML calibration file smaller than 64 KB.'
      return
    }
    saving = true
    try {
      const stem =
        file.name
          .replace(/\.toml$/i, '')
          .replace(/[^a-zA-Z0-9_-]/g, '-')
          .slice(0, 70) || 'custom'
      await ondevice({
        type: 'profile',
        id: stem === 'default' ? 'custom-default' : stem,
        toml: await file.text(),
      })
      draft = await preferencesApi.snapshot()
    } catch (reason) {
      message = reason instanceof Error ? reason.message : 'Could not import calibration.'
    } finally {
      saving = false
    }
  }
  async function changeWeights(key: string, value: number) {
    if (!weights || !Number.isFinite(value)) return
    saving = true
    try {
      await onweights({ ...weights, [key]: value })
      draft = await preferencesApi.snapshot()
    } catch (reason) {
      message = reason instanceof Error ? reason.message : 'Could not save weights.'
    } finally {
      saving = false
    }
  }
  async function loadHistory() {
    try {
      history = await preferencesApi.history()
    } catch (e) {
      message = e instanceof Error ? e.message : 'Could not load history.'
    }
  }
  const toggles: {
    key: keyof Pick<AutomationSettings, 'checkAfterMove' | 'checkAfterCycle'>
    label: string
    description: string
  }[] = [
    {
      key: 'checkAfterMove',
      label: 'Check position after each move',
      description: 'Read the phone to verify every placement.',
    },
    {
      key: 'checkAfterCycle',
      label: 'Check position after each cycle',
      description: 'Verify the resulting board before reading the next tray.',
    },
  ]
</script>

<dialog
  class="control-config settings-dialog unified-settings"
  bind:this={dialog}
  aria-labelledby={`${id}-title`}
  onclose={() => {
    code = ''
    pairingAddress = ''
  }}
  onclick={(event) => {
    if (event.target === dialog && !busy) dialog.close()
  }}
>
  <header class="settings-header">
    <div class="settings-heading">
      <span class="settings-heading-icon"><Icon name="settings" /></span>
      <div>
        <h2 id={`${id}-title`}>Settings</h2>
        <p>Workspace preferences</p>
      </div>
    </div>
    <form method="dialog">
      <button class="button control-settings" aria-label="Close settings" disabled={calibrating}
        ><Icon name="close" /></button
      >
    </form>
  </header>
  <div
    class="settings-tabs"
    role="tablist"
    aria-label="Settings sections"
    aria-orientation="vertical"
    tabindex="-1"
    onkeydown={(event) => {
      if (!['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key))
        return
      event.preventDefault()
      if (calibrating) return
      const index = sections.findIndex((section) => section.key === tab)
      const next =
        event.key === 'Home'
          ? 0
          : event.key === 'End'
            ? sections.length - 1
            : (index + (['ArrowUp', 'ArrowLeft'].includes(event.key) ? -1 : 1) + sections.length) %
              sections.length
      selectTab(sections[next]!.key)
      document.getElementById(`${id}-${sections[next]!.key}`)?.focus()
    }}
  >
    {#each sections as section (section.key)}
      <button
        id={`${id}-${section.key}`}
        class="settings-tab"
        disabled={calibrating}
        role="tab"
        type="button"
        aria-selected={tab === section.key}
        aria-controls={`${id}-${section.key}-panel`}
        tabindex={tab === section.key ? 0 : -1}
        onclick={() => selectTab(section.key)}>{section.label}</button
      >
    {/each}
    <p class="settings-nav-note" hidden={tab === 'calibration'}>
      {tab === 'calibration'
        ? 'Your progress stays here until you save.'
        : 'Changes save automatically.'}
    </p>
  </div>
  <div class="settings-body">
    <div
      id={`${id}-connection-panel`}
      role="tabpanel"
      tabindex="0"
      aria-labelledby={`${id}-connection`}
      hidden={tab !== 'connection'}
    >
      <div class="settings-connection">
        <div
          class={`settings-device-status${device?.error ? ' is-error' : device?.serial ? ' is-connected' : ''}`}
        >
          <span class="settings-status-mark"
            ><Icon name={device?.error ? 'info' : device?.serial ? 'check' : 'scan'} /></span
          >
          <div class="settings-copy">
            <strong
              >{device?.error
                ? 'Needs attention'
                : device?.busy
                  ? 'Working on your phone'
                  : device?.serial
                    ? 'Phone connected'
                    : 'Connect your phone'}</strong
            ><small role="status"
              >{device?.error ?? device?.message ?? 'Preparing local service…'}</small
            >{#if device?.serial}<span class="settings-serial">{device.serial}</span>{/if}
          </div>
        </div>
        <div class="settings-method" aria-label="Connection method">
          <button
            type="button"
            aria-pressed={draft?.connection.method !== 'wireless'}
            disabled={busy || !draft}
            onclick={() => {
              if (draft) {
                draft.connection.method = 'usb'
                void save()
              }
            }}>USB cable<span>Recommended</span></button
          ><button
            type="button"
            aria-pressed={draft?.connection.method === 'wireless'}
            disabled={busy || !draft}
            onclick={() => {
              if (draft) {
                draft.connection.method = 'wireless'
                void save()
              }
            }}>Wireless</button
          >
        </div>
        <div hidden={draft?.connection.method === 'wireless'} class="settings-connection-hint">
          <span class="settings-step-number"><Icon name="info" /></span>
          <p>
            Connect a data cable, enable <strong>USB debugging</strong> on your phone, and allow this
            computer.
          </p>
        </div>
        {#if draft && draft.connection.method === 'wireless'}
          <div class="settings-wireless">
            <div class="settings-section-heading">
              <h3>Wireless debugging</h3>
              <p>Keep your phone and computer on the same Wi-Fi network.</p>
            </div>
            <details class="settings-pairing" bind:open={pairingOpen}>
              <summary
                ><span
                  ><strong>Pair a new device</strong><small>First time on this computer only</small
                  ></span
                ><Icon name="arrow" /></summary
              >
              <form
                class="settings-pairing-body"
                onsubmit={(event) => {
                  event.preventDefault()
                  void wireless(true)
                }}
              >
                <p class="settings-description">
                  On your phone, open <strong
                    >Wireless debugging → Pair device with pairing code</strong
                  >. Keep that screen open while pairing.
                </p>
                <div class="settings-fields-grid">
                  <label class="settings-field"
                    ><span>Pairing IP address & port</span><input
                      class="settings-input"
                      aria-label="Pairing address"
                      placeholder="192.168.1.2:37123"
                      bind:value={pairingAddress}
                      disabled={busy}
                      required
                      autocomplete="off"
                      spellcheck={false}
                    /></label
                  >
                  <label class="settings-field"
                    ><span>Pairing code</span><input
                      class="settings-input"
                      aria-label="Pairing code"
                      placeholder="6-digit code"
                      bind:value={code}
                      disabled={busy}
                      required
                      inputmode="numeric"
                      pattern={'[0-9]{6}'}
                      minlength="6"
                      maxlength="6"
                      autocomplete="off"
                      spellcheck={false}
                    /></label
                  >
                </div>
                <button
                  class="button"
                  type="submit"
                  disabled={busy || !pairingAddress.trim() || !/^[0-9]{6}$/.test(code)}
                  >Pair device</button
                >
              </form>
            </details>
            <form
              class="settings-wireless-connect"
              onsubmit={(event) => {
                event.preventDefault()
                void wireless(false)
              }}
            >
              <label class="settings-field"
                ><span>Connection IP address & port</span><small
                  >From the main Wireless debugging screen. This port is different from the pairing
                  port.</small
                ><input
                  class="settings-input"
                  aria-label="Connection address"
                  placeholder="192.168.1.2:40567"
                  bind:value={draft.connection.wirelessAddress}
                  disabled={busy}
                  required
                  autocomplete="off"
                  spellcheck={false}
                /></label
              >
              <button
                class="button"
                type="submit"
                disabled={busy || !draft.connection.wirelessAddress.trim()}>Connect wireless</button
              >
            </form>
          </div>
        {/if}
        <div class="settings-device-picker">
          <div class="settings-picker-heading">
            <label for={`${id}-device`}>Choose a device</label><button
              class="button"
              type="button"
              disabled={busy || !device}
              onclick={() => void ondevice({ type: 'discover' })}
              ><Icon name="search" />Discover</button
            >
          </div>
          <select
            id={`${id}-device`}
            class="settings-select"
            aria-label="Device"
            disabled={busy || !draft}
            value={current}
            onchange={(e) => {
              if (draft) {
                draft.connection.serial = e.currentTarget.value
                void save()
              }
            }}
            ><option value="">Select phone</option
            >{#if current && !device?.devices.some((d) => d.serial === current)}<option
                value={current}>{current} · selected</option
              >{/if}{#each device?.devices ?? [] as d (d.serial)}<option
                value={d.serial}
                disabled={d.state !== 'device'}>{d.model || d.serial} · {d.state}</option
              >{/each}</select
          >{#if !device?.devices.length && !current}<p class="settings-field-help">
              No devices listed. Connect your phone, then Discover.
            </p>{/if}
        </div>
        <div class="settings-connect-actions">
          <button
            class="button button-primary"
            type="button"
            disabled={busy || !current}
            onclick={() =>
              void ondevice({ type: 'connect', serial: current, installHelper: install })}
            ><Icon name="check" />Connect</button
          ><button
            class="button"
            type="button"
            disabled={busy || !device?.serial}
            onclick={() => void ondevice({ type: 'disconnect' })}>Disconnect</button
          >
        </div>
        <details class="settings-advanced">
          <summary
            ><span
              ><strong>Device setup</strong><small
                >{device?.profileId === 'default'
                  ? 'Default · 1080 × 2400'
                  : (device?.profileId ?? 'Calibrated portrait layout')}</small
              ></span
            ><Icon name="arrow" /></summary
          >
          <div class="settings-advanced-body">
            <label class="settings-helper"
              ><input type="checkbox" bind:checked={install} disabled={busy} /><span
                class="settings-copy"
                ><strong>Install capture helper</strong><small
                  >Required once to read the game screen.</small
                ></span
              ></label
            ><label class="settings-field"
              ><span>Calibration profile</span><select
                class="settings-select"
                aria-label="Calibration"
                value={device?.profileId ?? 'default'}
                disabled={busy || !!device?.serial}
                onchange={async (e) => {
                  await ondevice({ type: 'selectProfile', id: e.currentTarget.value })
                  draft = await preferencesApi.snapshot()
                }}
                >{#each device?.profileIds ?? ['default'] as profileId (profileId)}<option
                    value={profileId}
                    >{profileId === 'default' ? 'Default · 1080 × 2400' : profileId}</option
                  >{/each}</select
              ><small>Choose a matching resolution and layout. Disconnect before changing.</small
              ></label
            ><label class="settings-field"
              ><span>Import calibration profile</span><input
                type="file"
                accept=".toml,text/plain"
                aria-label="Import calibration profile"
                disabled={busy || !!device?.serial}
                onchange={(e) => void importCalibration(e)}
              /><small
                >Hardware calibration only. Older profiles are accepted; scoring and combo sections
                are ignored.</small
              ></label
            >
          </div>
        </details>
        <button class="button" disabled={busy} onclick={() => selectTab('calibration')}
          >Calibrate or export HP profile</button
        >
        <p class="settings-next-step">
          <Icon name="info" />Once connected, Scan your phone. Enable Move device when you want to
          send movements.
        </p>
      </div>
    </div>
    <div
      id={`${id}-calibration-panel`}
      role="tabpanel"
      tabindex="0"
      aria-labelledby={`${id}-calibration`}
      hidden={tab !== 'calibration'}
    >
      <CalibrationPanel
        {device}
        serial={current}
        onconnection={() => selectTab('connection')}
        onclose={() => dialog.close()}
        {ondevice}
        onbusy={(value) => (calibrating = value)}
        onsaved={async () => {
          draft = await preferencesApi.snapshot()
        }}
      />
    </div>
    <div
      id={`${id}-automation-panel`}
      role="tabpanel"
      tabindex="0"
      aria-labelledby={`${id}-automation`}
      hidden={tab !== 'automation'}
    >
      <div class="settings-section-heading">
        <h3>Device control</h3>
        <p>Choose when JustBlast can send movements.</p>
      </div>
      <label class="settings-toggle settings-toggle-prominent"
        ><span class="settings-copy"
          ><strong>Move device</strong><small
            >Send movements to your connected phone. Preview is always available separately.</small
          ></span
        ><input
          type="checkbox"
          role="switch"
          aria-label="Move device"
          checked={draft?.automation.moveDevice ?? false}
          disabled={busy || !draft}
          onchange={(e) => {
            if (draft) {
              draft.automation.moveDevice = e.currentTarget.checked
              void save()
            }
          }}
        /></label
      >
      {#if draft?.automation.moveDevice}{#each toggles as toggle (toggle.key)}<label
            class="settings-toggle"
            ><span class="settings-copy"
              ><strong>{toggle.label}</strong><small>{toggle.description}</small></span
            ><input
              type="checkbox"
              role="switch"
              aria-label={toggle.label}
              checked={draft.automation[toggle.key]}
              disabled={busy}
              onchange={(e) => {
                if (draft) {
                  draft.automation[toggle.key] = e.currentTarget.checked
                  void save()
                }
              }}
            /></label
          >{/each}{/if}
      <div class="settings-group">
        <div class="settings-section-heading">
          <h3>Next cycle</h3>
          <p>Decide what happens after a completed phone cycle.</p>
        </div>
        <label class="settings-toggle"
          ><span class="settings-copy"
            ><strong>Continue automatically</strong><small
              >Read the next tray and calculate another plan.</small
            ></span
          ><input
            type="checkbox"
            role="switch"
            aria-label="Continue automatically"
            checked={draft?.automation.continuous ?? false}
            disabled={busy || !draft?.automation.moveDevice}
            onchange={(e) => {
              if (draft) {
                draft.automation.continuous = e.currentTarget.checked
                void save()
              }
            }}
          /></label
        >{#if draft?.automation.continuous}<label class="settings-field"
            ><span>Cycle limit (0 = until stopped)</span><input
              class="settings-input"
              type="number"
              min="0"
              max="100000"
              bind:value={draft.automation.cycleLimit}
              disabled={busy}
              onchange={() => void save()}
            /></label
          >{/if}
      </div>
    </div>
    <div
      id={`${id}-scoring-panel`}
      role="tabpanel"
      tabindex="0"
      aria-labelledby={`${id}-scoring`}
      hidden={tab !== 'scoring'}
    >
      <div class="settings-section-heading">
        <h3>Algorithm scoring</h3>
        <p>Set the weight of each factor when ranking complete plans. Changes apply immediately.</p>
      </div>
      <div class="settings-advanced-body">
        {#each ['points', 'combo', 'flexibility', 'fragmentation'] as key (key)}<label
            class="settings-field"
            ><span>{key}</span><input
              class="settings-input"
              type="number"
              min="0"
              step="1"
              aria-label={`${key} weight`}
              value={weights?.[key as keyof WeightsView] ?? 0}
              disabled={busy}
              onchange={(e) => {
                const value = Number(e.currentTarget.value)
                void changeWeights(key, value)
              }}
            /></label
          >{/each}
      </div>
    </div>
    <div
      id={`${id}-history-panel`}
      role="tabpanel"
      tabindex="0"
      aria-labelledby={`${id}-history`}
      hidden={tab !== 'history'}
    >
      <div class="settings-section-heading">
        <h3>Cycle history</h3>
        <p>Recent cycles and their results.</p>
      </div>
      <div class="settings-advanced-body">
        {#each [...history].reverse() as entry, index (`${entry.timestamp}-${index}`)}<p>
            {new Date(Number(entry.timestamp) * 1000).toLocaleString()} · {entry.source === 'auto'
              ? 'Legacy phone'
              : entry.source === 'manual'
                ? 'Legacy simulation'
                : entry.source} cycle {entry.cycle}
            · {entry.status}<br />{entry.message}
          </p>{/each}{#if !history.length}<p>No completed cycles yet.</p>{/if}
      </div>
    </div>
    {#if message && tab !== 'calibration'}<p class="settings-note" role="status">{message}</p>{/if}
  </div>
  <footer class="settings-footer" hidden={tab === 'calibration'}>
    <span
      ><Icon name="check" />{saving
        ? 'Updating…'
        : tab === 'calibration'
          ? 'Movement check places up to 3 shapes when requested'
          : draft?.automation.moveDevice
            ? 'Device control enabled'
            : 'Preview only · no phone movements'}</span
    ><button
      class="button button-primary"
      type="button"
      disabled={saving || calibrating}
      onclick={() => dialog.close()}>Done</button
    >
  </footer>
</dialog>
