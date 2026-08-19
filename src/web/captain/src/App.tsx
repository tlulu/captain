import { useState, useEffect } from 'react'
import './App.css'
import type { PodInfo } from '../../../../bindings/PodInfo'
import type { GetPodsResponse } from '../../../../bindings/GetPodsResponse'
import DeploymentTable from './components/DeploymentTable'

function App() {
  const [pods, setPods] = useState<PodInfo[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [fetchError, setFetchError] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);

  const fetchPodStatus = async () => {
    try {
      const response = await fetch('/api/pods');
      if (!response.ok) {
        throw new Error('Failed to fetch pod status');
      }
      const data: GetPodsResponse = await response.json();
      setPods(data.pods);
      setFetchError(null);
    } catch (err: any) {
      setFetchError(err.message || 'An error occurred');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchPodStatus();
    const intervalId = setInterval(fetchPodStatus, 500);
    return () => clearInterval(intervalId);
  }, []);

  // Find unique images in all pods and sort them (oldest first)
  const uniqueImages = Array.from(new Set(pods.map(p => p.image))).sort((a, b) => {
    const timeA = new Date(pods.find(p => p.image === a)?.created_at || 0).getTime();
    const timeB = new Date(pods.find(p => p.image === b)?.created_at || 0).getTime();
    return timeA - timeB;
  });

  console.assert(uniqueImages.length <= 2, "Expected at most 2 unique images in the pods list");

  const currentDeployment = uniqueImages[0] ? pods.filter(p => p.image === uniqueImages[0]) : [];
  const newDeployment = uniqueImages[1] ? pods.filter(p => p.image === uniqueImages[1]) : [];

  return (
    <div className="p-8 bg-slate-50 min-h-screen flex flex-col items-center text-center gap-6">
      <h1 className="text-3xl font-bold text-slate-800">SS Captain</h1>

      {loading && <p className="text-slate-500">Loading pods...</p>}

      {actionError && (
        <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-2 rounded-md max-w-3xl w-full flex justify-between items-center text-sm shadow-sm">
          <span className="text-left font-medium break-all">{actionError}</span>
          <button
            onClick={() => setActionError(null)}
            className="ml-4 font-bold text-red-400 hover:text-red-600 transition-colors shrink-0"
          >
            ✕
          </button>
        </div>
      )}

      {fetchError && !actionError && (
        <p className="text-red-500 text-sm">Fetch Error: {fetchError}</p>
      )}

      <div className="flex flex-col w-fit items-stretch gap-6 mx-auto">
        {/* Current Deployment */}
        {currentDeployment.length > 0 && (
          <DeploymentTable
            pods={currentDeployment}
            onError={setActionError}
            isCanary={currentDeployment.some(p => p.is_canary)}
          />
        )}

        {/* New Deployment */}
        {newDeployment.length > 0 && (
          <>
            <div className="text-slate-400 flex items-center justify-center my-4 drop-shadow">
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round" className="w-8 h-8">
                <line x1="12" y1="4" x2="12" y2="16" />
                <path d="M7 12l5 5 5-5" />
              </svg>
            </div>
            <DeploymentTable
              pods={newDeployment}
              onError={setActionError}
              isCanary={newDeployment.some(p => p.is_canary)}
            />
          </>
        )}
      </div>
    </div>
  );
}

export default App;
