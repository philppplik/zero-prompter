import { mount } from 'svelte';
import '@fontsource-variable/dm-sans'; // UI (tweakcn "Minimal Neutral")
import '@fontsource-variable/geist-mono'; // UI numbers / kbd
import '@fontsource-variable/inter'; // prompter font choice
import '@fontsource-variable/jetbrains-mono'; // prompter font choice
import '@fontsource/atkinson-hyperlegible/400.css';
import '@fontsource/atkinson-hyperlegible/700.css';
import './app.css';
import App from './components/App.svelte';
import { applyUiTokens } from './lib/tokens';

applyUiTokens();

export default mount(App, { target: document.getElementById('app')! });
