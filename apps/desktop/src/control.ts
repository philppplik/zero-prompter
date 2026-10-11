import { mount } from 'svelte';
import '@fontsource-variable/dm-sans'; // UI (tweakcn "Minimal Neutral")
import '@fontsource-variable/geist-mono'; // UI numbers / kbd
import './fonts'; // prompter fonts
import './app.css';
import App from './components/App.svelte';
import { applyUiTokens } from './lib/tokens';

applyUiTokens();

export default mount(App, { target: document.getElementById('app')! });
