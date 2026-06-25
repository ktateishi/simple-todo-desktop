export interface VoiceCallbacks {
  onResult: (text: string) => void;
  onError:  (msg: string)  => void;
  onEnd:    ()             => void;
}

export function createVoiceInput() {
  let rec: any = null;
  let active = false;

  const isSupported = () =>
    typeof window !== 'undefined' &&
    ('SpeechRecognition' in window || 'webkitSpeechRecognition' in window);

  const start = (cb: VoiceCallbacks) => {
    if (!isSupported()) { cb.onError('この環境では音声入力を利用できません'); return; }
    const SR = (window as any).SpeechRecognition ?? (window as any).webkitSpeechRecognition;
    rec = new SR();
    rec.lang = 'ja-JP';
    rec.interimResults = false;
    rec.maxAlternatives = 1;
    rec.onresult = (e: any) => cb.onResult(e.results[0][0].transcript);
    rec.onerror  = (e: any) => { active = false; cb.onError(e.error); };
    rec.onend    = ()       => { active = false; cb.onEnd(); };
    rec.start();
    active = true;
  };

  const stop = () => { rec?.stop(); active = false; };

  return { start, stop, isSupported, isActive: () => active };
}

// Parse `#tagname` tokens from a raw input string
export function parseInput(raw: string): { title: string; tagNames: string[] } {
  const tagNames: string[] = [];
  const title = raw.replace(/#(\S+)/g, (_, name) => { tagNames.push(name); return ''; }).trim();
  return { title, tagNames };
}
