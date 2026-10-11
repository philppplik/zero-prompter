import { mount } from 'svelte';
import './fonts'; // prompter fonts
import './app.css';
import Talent from './components/Talent.svelte';
import { applyUiTokens } from './lib/tokens';
import { store } from './lib/store.svelte';

applyUiTokens();
store.init().catch((e) => console.error('zt: talent init failed', e));

export default mount(Talent, { target: document.getElementById('app')! });
