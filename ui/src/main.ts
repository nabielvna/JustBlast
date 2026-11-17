import { mount } from 'svelte'
import App from './app/App.svelte'
import './styles/fonts.css'
import './styles/dashboard.css'
import './styles/settings.css'
import './styles/calibration.css'
import './styles/device-controls.css'
import './styles/board-motion.css'
import './styles/workspace.css'

const target = document.getElementById('app')
if (!target) throw new Error('Missing application root')

mount(App, { target })
import './styles/buttons.css'
