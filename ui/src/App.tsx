import { BrowserProvider, useBrowser, createBrowserState } from './domain';
import { AgentProvider, useAgent } from './domain';
import { Toolbar } from './components/Toolbar';
import { AgentPanel } from './components/AgentPanel';

function Main() {
  const browser = useBrowser();
  const agentState = useAgent();

  return (
    <div className="ba-app">
      <Toolbar
        browser={browser}
        onNavigate={(url, appId) => {
          console.log('Navigate:', url, appId);
        }}
      />

      <div className="ba-content">
        <AgentPanel
          agent={agentState.state}
          onSubmit={(text) => {
            console.log('Submit task:', text);
            return Promise.resolve('done');
          }}
        />
        <div className="ba-viewport">
          <p>Viewport content</p>
        </div>
      </div>
    </div>
  );
}

export function App() {
  return (
    <BrowserProvider initial={createBrowserState()} onBridgeReady={() => undefined}>
      <AgentProvider>
        <Main />
      </AgentProvider>
    </BrowserProvider>
  );
}
