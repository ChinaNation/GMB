import { Routes, Route, useLocation } from 'react-router-dom'
import { Suspense, lazy, useEffect } from 'react'
import Header from './components/Header'
import Footer from './components/Footer'
import Home from './pages/Home'
import About from './pages/About'
import Technology from './pages/Technology'
import Tokenomics from './pages/Tokenomics'
import Governance from './pages/Governance'
import Ecosystem from './pages/Ecosystem'

const Whitepaper = lazy(() => import('./pages/Whitepaper'))

function ScrollToTop() {
  const { pathname } = useLocation()
  useEffect(() => { window.scrollTo(0, 0) }, [pathname])
  return null
}

export default function App() {
  return (
    <>
      <ScrollToTop />
      <Header />
      <main className="pt-[72px]">
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/about" element={<About />} />
          <Route path="/technology" element={<Technology />} />
          <Route path="/tokenomics" element={<Tokenomics />} />
          <Route path="/governance" element={<Governance />} />
          <Route
            path="/whitepaper"
            element={(
              <Suspense fallback={<div className="min-h-screen bg-navy-950 px-6 py-16 text-slate-300">白皮书加载中...</div>}>
                <Whitepaper />
              </Suspense>
            )}
          />
          <Route path="/ecosystem" element={<Ecosystem />} />
        </Routes>
      </main>
      <Footer />
    </>
  )
}
