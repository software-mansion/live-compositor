import clsx from 'clsx';
import {
  FaBook,
  FaCode,
  FaDocker,
  FaFile,
  FaGithub,
  FaLink,
  FaPuzzlePiece,
  FaRust,
} from 'react-icons/fa6';
import { GiFeather } from 'react-icons/gi';
import { MdLiveTv } from 'react-icons/md';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import Heading from '@theme/Heading';

import styles from './index.module.css';
import { PropsWithChildren } from 'react';
import { IconType } from 'react-icons';
import MembraneLogo from '@site/static/img/membrane-logo.svg';
import SwmLogo from '@site/static/img/swm-logo.svg';
import ComposingImg from '@site/static/img/how_it_works.png';
import WebGpuLogoDark from '@site/static/img/webgpu-dark.svg';
import WebGpuLogoLight from '@site/static/img/webgpu-light.svg';
import VideoConferencingImg from '@site/static/img/videoconferencing.jpg';
import StreamingImg from '@site/static/img/streaming.jpg';
import BroadcastingImg from '@site/static/img/broadcasting.jpg';
import { useColorMode } from '@docusaurus/theme-common';
import TypewriterComponent from 'typewriter-effect';
import ExampleScene from './example_scene';

function HomepageHeader() {
  return (
    <div className={clsx('container', styles.firstSection)}>
      <div className="row">
        <div className="col col--6">
          <div className={styles.shadow}></div>
          <div className="container">
            <Heading as="h1" style={{ fontSize: 55 }}>
              <span className="text--primary">Mix video and audio</span>
              <div className={styles.typewriter}>
                <TypewriterComponent
                  options={{
                    strings: ['in real-time.', 'using code.', 'with low-latency.'],
                    autoStart: true,
                    loop: true,
                    deleteSpeed: 30,
                    delay: 80,
                  }}
                />
              </div>
            </Heading>
            <p className={styles.sectionSubheading}>
              Open-source media server for real-time, low-latency, programmable video and audio
              mixing.
            </p>
            <div className="row margin-bottom--md">
              <Link
                className={clsx(
                  'button button--primary button--lg col col-6 margin--sm',
                  styles.hoverPrimary
                )}
                to="/docs/intro">
                <FaBook style={{ marginRight: 3 }} />
                Docs
              </Link>
              <Link
                className={clsx(
                  'button button--secondary button--outline button--lg col col-6 margin--sm',
                  styles.hoverSecondary
                )}
                to="https://github.com/membraneframework/live_compositor">
                <FaGithub style={{ marginRight: 5 }} />
                View on GitHub
              </Link>
            </div>
          </div>
        </div>
        <div className="col col--6">
          <div
            style={{
              borderRadius: '30px',
              overflow: 'hidden',
            }}>
            <video
              loop
              autoPlay
              muted
              src="https://github.com/membraneframework/live_compositor/assets/104033489/e6f5ba7c-ab05-4935-a42a-bc28c42fc895"
              poster="/img/demo_poster.jpg"
              style={{ width: '100%', display: 'block' }}
            />
          </div>
        </div>
      </div>
    </div>
  );
}

function HowItWorks() {
  return (
    <div className="container margin-top--md">
      <Heading as="h1" className="margin-bottom--md text--center text--primary">
        How it works?
      </Heading>
      <p className={clsx('text--center', styles.sectionSubheading)}>
        Send inputs as RTP streams or MP4 files. Configure mixing with HTTP requests. Get the mixed
        streams as RTP.
        <br />
      </p>
      <div className="row">
        <div className="col col--6">
          <img src={ComposingImg} alt="Composing" />
        </div>
        <div className={clsx('col col--6', styles.sceneExample)}>
          <ExampleScene />
        </div>
      </div>
    </div>
  );
}

function Applications() {
  return (
    <div className="container margin-top--md">
      <Heading as="h1" className="margin-bottom--md text--center text--primary">
        Applications
      </Heading>
      <p className={clsx('text--center', styles.sectionSubheading)}>
        Use LiveCompositor for video conferencing, live-streaming, broadcasting, and more.
      </p>
      <div className="row">
        <ApplicationCard
          title="Video conferencing"
          subtitle="Stream or record video conferences"
          img={VideoConferencingImg}
        />
        <ApplicationCard title="Broadcasting" subtitle="Compose broadcasts" img={BroadcastingImg} />
        <ApplicationCard
          title="Live-streaming"
          subtitle="Create awesome live-streams"
          img={StreamingImg}
        />
      </div>
    </div>
  );
}

type ApplicationCardProps = {
  title: string;
  subtitle: string;
  img: any;
};

function ApplicationCard(props: ApplicationCardProps) {
  return (
    <div className={clsx('card', styles.card, styles.hoverPrimary)}>
      <div className="text--primary">
        <Heading as="h2" style={{ textAlign: 'center', margin: 0 }}>
          {props.title}
        </Heading>
      </div>
      <p className={clsx('text--center', styles.sectionSubheading)} style={{ margin: 0 }}>
        {props.subtitle}
      </p>
      <div className="card__body">
        <img src={props.img} alt={props.title} />
      </div>
    </div>
  );
}

function VisionCards() {
  return (
    <div className="container margin-top--md">
      <Heading as="h1" className="margin-bottom--md text--center text--primary">
        Vision
      </Heading>
      <p className={clsx('text--center', styles.sectionSubheading)}>
        Make mixing live streams as simple as building a website.
      </p>
      <div className="row">
        <VisionCard title="Simple declarative API" icon={FaCode}>
          <p className="padding--md">
            Mixing is specified using simple component tree, easy to pick up for anyone familiar
            with web development.
          </p>
        </VisionCard>
        <VisionCard title="Easy integration" icon={FaLink}>
          <p className="padding--md">
            API is language agnostic. You can leverage tech stack of your choice and easily
            integrate it with your own solutions.
          </p>
        </VisionCard>
        <VisionCard title="Real-time performance" icon={FaRust}>
          <p className="padding--md">
            LiveCompositor focus on real-time processing and low-latency. It's implemented in Rust
            and use WebGPU for rendering.
          </p>
        </VisionCard>
      </div>
    </div>
  );
}

type VisionCardProps = {
  title: string;
  icon: IconType;
};

function VisionCard(props: PropsWithChildren<VisionCardProps>) {
  const Icon = props.icon;
  return (
    <div className={clsx('card', styles.card, styles.hoverPrimary)}>
      <div style={{ display: 'flex', justifyContent: 'center' }}>
        <Icon className={styles.icon} />
      </div>
      <div className="card__header">
        <Heading as="h2" style={{ textAlign: 'center' }}>
          {props.title}
        </Heading>
      </div>
      <div className="card__body">{props.children}</div>
    </div>
  );
}

type FeatureProps = {
  text: string;
  secondaryText?: string;
  image: any;
};

function Feature(props: PropsWithChildren<FeatureProps>) {
  const text = (
    <div className="col">
      <Heading as="h2" className="margin-top--sm">
        {props.text}
      </Heading>
      <p>{props.secondaryText}</p>
    </div>
  );
  return (
    <div className="row margin-right--lg margin-left--lg" style={{ alignItems: 'center' }}>
      {props.image}
      <div className="col">
        {text}
        {props.children}
      </div>
    </div>
  );
}

function Features() {
  const { colorMode } = useColorMode();

  const wgpuLogo =
    colorMode === 'dark' ? (
      <WebGpuLogoDark className={styles.featureIcon} width={80} height={80} />
    ) : (
      <WebGpuLogoLight className={styles.featureIcon} width={80} height={80} />
    );

  return (
    <div className="container margin-top--lg margin-bottom--md">
      <Heading as="h1" className="margin-bottom--md text--center text--primary">
        Capabilities
      </Heading>
      <p className={clsx('text--center', styles.sectionSubheading)}>
        Simple, powerful, fast. Pick three.
      </p>

      <Feature
        text="Built-in components"
        secondaryText="Use built-in components to process streams, render text, images, GIFs or websites, and combine them into output streams."
        image={<FaPuzzlePiece className={styles.featureIcon} style={{ fontSize: 80 }} />}
      />

      <Feature
        text="Highly customizable"
        secondaryText="Create custom effects with WGSL shaders and directly leverage GPU parallel processing capabilities."
        image={wgpuLogo}
      />

      <Feature
        text="Smooth transitions"
        secondaryText="Smoothly transition layouts with animations using built-in transition mechanism."
        image={<GiFeather className={styles.featureIcon} style={{ fontSize: 80 }} />}
      />

      <Feature
        text="Real-time processing"
        secondaryText="Process live video streams in real-time with low latency. Blazingly fast processing makes video conferencing, live-streaming, and broadcasting an everyday bread and butter for Live Compositor."
        image={<MdLiveTv className={styles.featureIcon} style={{ fontSize: 80 }} />}
      />

      <Feature
        text="Easy deployment"
        secondaryText="Docker images, nix configs, and guides are made to make deployment as easy as possible."
        image={<FaDocker className={styles.featureIcon} style={{ fontSize: 80 }} />}
      />
    </div>
  );
}

function IntegrationMembranePlugin() {
  return (
    <div className="row" style={{ justifyContent: 'center' }}>
      <MembraneLogo
        width={200}
        height={200}
        className="margin--lg"
        style={{ alignSelf: 'center' }}
      />
      <div className="col">
        <div className="card">
          <div className="card__header">
            <Heading as="h3" className="margin-top--sm">
              Membrane plugin
            </Heading>
          </div>
          <div className="card__body container">
            <p>
              Membrane is a developer-friendly multimedia framework for Elixir. You can easily add
              video composing functionality into your multimedia pipeline using Membrane Live
              Compositor Plugin.
            </p>
            <p>For more, see:</p>
            <p>
              <FaFile style={{ fontSize: 15, marginRight: 7 }} />
              Documentation -{' '}
              <Link href="/docs/intro#membrane-framework-plugin">
                Get started with Membrane plugin
              </Link>
            </p>
            <p>
              <FaGithub style={{ fontSize: 15, marginRight: 7 }} />
              GitHub repository -{' '}
              <Link href="https://github.com/membraneframework/membrane_live_compositor_plugin">
                membraneframework/membrane_live_compositor_plugin
              </Link>
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}

function Integrations() {
  return (
    <div className="container">
      <Heading as="h1" className="margin-bottom--md text--center text--primary">
        Integrations
      </Heading>
      <IntegrationMembranePlugin />
    </div>
  );
}

function ContactUs() {
  return (
    <div className="container margin-bottom--lg">
      <div className="card container">
        <div className="row" style={{ justifyContent: 'center' }}>
          <SwmLogo width={220} className="margin--lg" style={{ alignSelf: 'center' }} />
          <div className="col">
            <div className="card__header margin-top--md">
              <Heading as="h1">
                <span className="text--primary">Contact</span> us
              </Heading>
            </div>
            <div className="card__body">
              <p className={styles.sectionSubheading}>
                LiveCompositor is free and open-source. It's created by Software Mansion - a
                software company that is specialized in building tools for developers. At Software
                Mansion, we work on multiple multimedia projects, like Membrane Framework, Elixir
                WebRTC, FishJam, and more. We also work on custom solutions for clients. Contact us
                and create something together.
              </p>
              <div className="row" style={{ justifyContent: 'end' }}>
                <Link
                  className={clsx(
                    'button',
                    'button--primary',
                    'button--lg',
                    'margin--sm',
                    styles.contactButton,
                    styles.hoverPrimary
                  )}
                  to="https://membrane.stream/contact">
                  Contact us
                </Link>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default function Home(): JSX.Element {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout
      title={siteConfig.title}
      description="Tool for real-time video processing / transforming / composing">
      <HomepageHeader />
      <div className={styles.sectionSeparator} />
      <HowItWorks />
      <div className={styles.sectionSeparator} />
      <Applications />
      <div className={styles.sectionSeparator} />
      <VisionCards />
      <div className={styles.sectionSeparator} />
      <Features />
      <div className={styles.sectionSeparator} />
      <Integrations />
      <div className={styles.sectionSeparator} />
      <ContactUs />
    </Layout>
  );
}
